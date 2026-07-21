#![allow(non_snake_case)]
//! `huffman.cpp` — the adaptive Huffman coder (Sayood's algorithm) used by the
//! net-channel and the message bit-stream layer.
//!
//! Raven's file-static `bloc` bit cursor (rule B3: no `static mut`) is threaded
//! as an explicit `&mut i32` through the helpers that share it within one
//! compress/decompress pass; the public `Huff_putBit`/`Huff_getBit` and the
//! `Huff_offset*` entry points load it from / store it to their `offset`
//! out-param exactly as the oracle does.
//!
//! Source: `oracle/codemp/qcommon/huffman.cpp`

use core::ffi::{c_char, c_int};
use core::ptr;

use super::huff_t::huff_t;
use super::huffman_consts::{INTERNAL_NODE, NYT};
use super::huffman_t::huffman_t;
use super::nodetype::node_t;

use mp_qshared::common::mp::qcommon::msg_t::msg_t;

/// Raven `Huff_putBit`.
///
/// Source: `oracle/codemp/qcommon/huffman.cpp:12-19`
pub unsafe fn Huff_putBit(bit: c_int, fout: *mut u8, offset: &mut c_int) {
    add_bit(bit as c_char, fout, offset);
}

/// Raven `Huff_getBit`.
///
/// Source: `oracle/codemp/qcommon/huffman.cpp:22-29`
pub unsafe fn Huff_getBit(fin: *mut u8, offset: &mut c_int) -> c_int {
    get_bit(fin, offset)
}

/// Raven `add_bit` — buffered bit append (uses the shared `bloc` cursor).
///
/// Source: `oracle/codemp/qcommon/huffman.cpp:32-38`
unsafe fn add_bit(bit: c_char, fout: *mut u8, bloc: &mut c_int) {
    if (*bloc & 7) == 0 {
        *fout.add((*bloc >> 3) as usize) = 0;
    }
    *fout.add((*bloc >> 3) as usize) |= (bit << (*bloc & 7)) as u8;
    *bloc += 1;
}

/// Raven `get_bit` — buffered bit fetch (uses the shared `bloc` cursor).
///
/// Source: `oracle/codemp/qcommon/huffman.cpp:41-46`
unsafe fn get_bit(fin: *mut u8, bloc: &mut c_int) -> c_int {
    let t = ((*fin.add((*bloc >> 3) as usize) as c_int) >> (*bloc & 7)) & 0x1;
    *bloc += 1;
    t
}

/// Raven `get_ppnode`.
///
/// Source: `oracle/codemp/qcommon/huffman.cpp:48-57`
unsafe fn get_ppnode(huff: *mut huff_t) -> *mut *mut node_t {
    if (*huff).freelist.is_null() {
        let idx = (*huff).blocPtrs;
        (*huff).blocPtrs += 1;
        (*huff).nodePtrs.as_mut_ptr().add(idx as usize)
    } else {
        let tppnode = (*huff).freelist;
        (*huff).freelist = *tppnode as *mut *mut node_t;
        tppnode
    }
}

/// Raven `free_ppnode`.
///
/// Source: `oracle/codemp/qcommon/huffman.cpp:59-62`
unsafe fn free_ppnode(huff: *mut huff_t, ppnode: *mut *mut node_t) {
    *ppnode = (*huff).freelist as *mut node_t;
    (*huff).freelist = ppnode;
}

/// Raven `swap` — swap the location of two nodes in the tree.
///
/// Source: `oracle/codemp/qcommon/huffman.cpp:65-93`
unsafe fn swap(huff: *mut huff_t, node1: *mut node_t, node2: *mut node_t) {
    let par1 = (*node1).parent;
    let par2 = (*node2).parent;

    if !par1.is_null() {
        if (*par1).left == node1 {
            (*par1).left = node2;
        } else {
            (*par1).right = node2;
        }
    } else {
        (*huff).tree = node2;
    }

    if !par2.is_null() {
        if (*par2).left == node2 {
            (*par2).left = node1;
        } else {
            (*par2).right = node1;
        }
    } else {
        (*huff).tree = node1;
    }

    (*node1).parent = par2;
    (*node2).parent = par1;
}

/// Raven `swaplist` — swap two nodes in the linked list (update ranks).
///
/// Source: `oracle/codemp/qcommon/huffman.cpp:96-125`
unsafe fn swaplist(node1: *mut node_t, node2: *mut node_t) {
    let mut par1 = (*node1).next;
    (*node1).next = (*node2).next;
    (*node2).next = par1;

    par1 = (*node1).prev;
    (*node1).prev = (*node2).prev;
    (*node2).prev = par1;

    if (*node1).next == node1 {
        (*node1).next = node2;
    }
    if (*node2).next == node2 {
        (*node2).next = node1;
    }
    if !(*node1).next.is_null() {
        (*(*node1).next).prev = node1;
    }
    if !(*node2).next.is_null() {
        (*(*node2).next).prev = node2;
    }
    if !(*node1).prev.is_null() {
        (*(*node1).prev).next = node1;
    }
    if !(*node2).prev.is_null() {
        (*(*node2).prev).next = node2;
    }
}

/// Raven `increment` — do the increments.
///
/// Source: `oracle/codemp/qcommon/huffman.cpp:128-163`
unsafe fn increment(huff: *mut huff_t, node: *mut node_t) {
    if node.is_null() {
        return;
    }

    if !(*node).next.is_null() && (*(*node).next).weight == (*node).weight {
        let lnode = *(*node).head;
        if lnode != (*node).parent {
            swap(huff, lnode, node);
        }
        swaplist(lnode, node);
    }
    if !(*node).prev.is_null() && (*(*node).prev).weight == (*node).weight {
        *(*node).head = (*node).prev;
    } else {
        *(*node).head = ptr::null_mut();
        free_ppnode(huff, (*node).head);
    }
    (*node).weight += 1;
    if !(*node).next.is_null() && (*(*node).next).weight == (*node).weight {
        (*node).head = (*(*node).next).head;
    } else {
        (*node).head = get_ppnode(huff);
        *(*node).head = node;
    }
    if !(*node).parent.is_null() {
        increment(huff, (*node).parent);
        if (*node).prev == (*node).parent {
            swaplist(node, (*node).parent);
            if *(*node).head == node {
                *(*node).head = (*node).parent;
            }
        }
    }
}

/// Raven `Huff_addRef`.
///
/// Source: `oracle/codemp/qcommon/huffman.cpp:166-234`
pub unsafe fn Huff_addRef(huff: *mut huff_t, ch: u8) {
    let ch = ch as usize;
    if (*huff).loc[ch].is_null() {
        // if this is the first transmission of this node
        let tnode = (*huff).nodeList.as_mut_ptr().add((*huff).blocNode as usize);
        (*huff).blocNode += 1;
        let tnode2 = (*huff).nodeList.as_mut_ptr().add((*huff).blocNode as usize);
        (*huff).blocNode += 1;

        (*tnode2).symbol = INTERNAL_NODE;
        (*tnode2).weight = 1;
        (*tnode2).next = (*(*huff).lhead).next;
        if !(*(*huff).lhead).next.is_null() {
            (*(*(*huff).lhead).next).prev = tnode2;
            if (*(*(*huff).lhead).next).weight == 1 {
                (*tnode2).head = (*(*(*huff).lhead).next).head;
            } else {
                (*tnode2).head = get_ppnode(huff);
                *(*tnode2).head = tnode2;
            }
        } else {
            (*tnode2).head = get_ppnode(huff);
            *(*tnode2).head = tnode2;
        }
        (*(*huff).lhead).next = tnode2;
        (*tnode2).prev = (*huff).lhead;

        (*tnode).symbol = ch as c_int;
        (*tnode).weight = 1;
        (*tnode).next = (*(*huff).lhead).next;
        if !(*(*huff).lhead).next.is_null() {
            (*(*(*huff).lhead).next).prev = tnode;
            if (*(*(*huff).lhead).next).weight == 1 {
                (*tnode).head = (*(*(*huff).lhead).next).head;
            } else {
                // this should never happen
                (*tnode).head = get_ppnode(huff);
                *(*tnode).head = tnode2;
            }
        } else {
            // this should never happen
            (*tnode).head = get_ppnode(huff);
            *(*tnode).head = tnode;
        }
        (*(*huff).lhead).next = tnode;
        (*tnode).prev = (*huff).lhead;
        (*tnode).left = ptr::null_mut();
        (*tnode).right = ptr::null_mut();

        if !(*(*huff).lhead).parent.is_null() {
            if (*(*(*huff).lhead).parent).left == (*huff).lhead {
                // lhead is guaranteed to be the NYT
                (*(*(*huff).lhead).parent).left = tnode2;
            } else {
                (*(*(*huff).lhead).parent).right = tnode2;
            }
        } else {
            (*huff).tree = tnode2;
        }

        (*tnode2).right = tnode;
        (*tnode2).left = (*huff).lhead;

        (*tnode2).parent = (*(*huff).lhead).parent;
        (*(*huff).lhead).parent = tnode2;
        (*tnode).parent = tnode2;

        (*huff).loc[ch] = tnode;

        increment(huff, (*tnode2).parent);
    } else {
        increment(huff, (*huff).loc[ch]);
    }
}

/// Raven `Huff_Receive` — get a symbol.
///
/// Source: `oracle/codemp/qcommon/huffman.cpp:236-249`
unsafe fn Huff_Receive(
    mut node: *mut node_t,
    ch: &mut c_int,
    fin: *mut u8,
    bloc: &mut c_int,
) -> c_int {
    while !node.is_null() && (*node).symbol == INTERNAL_NODE {
        if get_bit(fin, bloc) != 0 {
            node = (*node).right;
        } else {
            node = (*node).left;
        }
    }
    if node.is_null() {
        return 0;
        // Com_Error(ERR_DROP, "Illegal tree!\n");
    }
    *ch = (*node).symbol;
    *ch
}

/// Raven `Huff_offsetReceive` — get a symbol.
///
/// Source: `oracle/codemp/qcommon/huffman.cpp:252-269`
pub unsafe fn Huff_offsetReceive(
    mut node: *mut node_t,
    ch: &mut c_int,
    fin: *mut u8,
    offset: &mut c_int,
) {
    let mut bloc = *offset;
    while !node.is_null() && (*node).symbol == INTERNAL_NODE {
        if get_bit(fin, &mut bloc) != 0 {
            node = (*node).right;
        } else {
            node = (*node).left;
        }
    }
    if node.is_null() {
        *ch = 0;
        return;
        // Com_Error(ERR_DROP, "Illegal tree!\n");
    }
    *ch = (*node).symbol;
    *offset = bloc;
}

/// Raven `send` — send the prefix code for this node.
///
/// Source: `oracle/codemp/qcommon/huffman.cpp:271-282`
unsafe fn send(node: *mut node_t, child: *mut node_t, fout: *mut u8, bloc: &mut c_int) {
    if !(*node).parent.is_null() {
        send((*node).parent, node, fout, bloc);
    }
    if !child.is_null() {
        if (*node).right == child {
            add_bit(1, fout, bloc);
        } else {
            add_bit(0, fout, bloc);
        }
    }
}

/// Raven `Huff_transmit` — send a symbol.
///
/// Source: `oracle/codemp/qcommon/huffman.cpp:285-296`
unsafe fn Huff_transmit(huff: *mut huff_t, ch: c_int, fout: *mut u8, bloc: &mut c_int) {
    if (*huff).loc[ch as usize].is_null() {
        // node_t hasn't been transmitted, send a NYT, then the symbol
        Huff_transmit(huff, NYT, fout, bloc);
        for i in (0..=7).rev() {
            add_bit(((ch >> i) & 0x1) as c_char, fout, bloc);
        }
    } else {
        send((*huff).loc[ch as usize], ptr::null_mut(), fout, bloc);
    }
}

/// Raven `Huff_offsetTransmit`.
///
/// Source: `oracle/codemp/qcommon/huffman.cpp:298-302`
pub unsafe fn Huff_offsetTransmit(huff: *mut huff_t, ch: c_int, fout: *mut u8, offset: &mut c_int) {
    let mut bloc = *offset;
    send((*huff).loc[ch as usize], ptr::null_mut(), fout, &mut bloc);
    *offset = bloc;
}

/// Raven `Huff_Decompress`.
///
/// Source: `oracle/codemp/qcommon/huffman.cpp:304-356`
pub unsafe fn Huff_Decompress(mbuf: *mut msg_t, offset: c_int) {
    let mut seq = [0u8; 65536];
    let mut huff: huff_t = core::mem::zeroed();

    let size = (*mbuf).cursize - offset;
    let buffer = (*mbuf).data.add(offset as usize);

    if size <= 0 {
        return;
    }

    // Initialize the tree & list with the NYT node
    let nyt = huff.nodeList.as_mut_ptr().add(huff.blocNode as usize);
    huff.blocNode += 1;
    huff.tree = nyt;
    huff.lhead = nyt;
    huff.ltail = nyt;
    huff.loc[NYT as usize] = nyt;
    (*huff.tree).symbol = NYT;
    (*huff.tree).weight = 0;
    (*huff.lhead).next = ptr::null_mut();
    (*huff.lhead).prev = ptr::null_mut();
    (*huff.tree).parent = ptr::null_mut();
    (*huff.tree).left = ptr::null_mut();
    (*huff.tree).right = ptr::null_mut();

    let mut cch = (*buffer.add(0) as c_int) * 256 + (*buffer.add(1) as c_int);
    // don't overflow with bad messages
    if cch > (*mbuf).maxsize - offset {
        cch = (*mbuf).maxsize - offset;
    }
    let mut bloc: c_int = 16;

    for j in 0..cch {
        let mut ch: c_int = 0;
        // don't overflow reading from the messages
        if (bloc >> 3) > size {
            seq[j as usize] = 0;
            break;
        }
        Huff_Receive(huff.tree, &mut ch, buffer, &mut bloc); // Get a character
        if ch == NYT {
            // We got a NYT, get the symbol associated with it
            ch = 0;
            for _ in 0..8 {
                ch = (ch << 1) + get_bit(buffer, &mut bloc);
            }
        }

        seq[j as usize] = ch as u8; // Write symbol

        Huff_addRef(&mut huff, ch as u8); // Increment node
    }
    (*mbuf).cursize = cch + offset;
    ptr::copy_nonoverlapping(
        seq.as_ptr(),
        (*mbuf).data.add(offset as usize),
        cch as usize,
    );
}

/// Raven `Huff_Compress`.
///
/// Source: `oracle/codemp/qcommon/huffman.cpp:360-395`
pub unsafe fn Huff_Compress(mbuf: *mut msg_t, offset: c_int) {
    let mut seq = [0u8; 65536];
    let mut huff: huff_t = core::mem::zeroed();

    let size = (*mbuf).cursize - offset;
    let buffer = (*mbuf).data.add(offset as usize);

    if size <= 0 {
        return;
    }

    // Add the NYT (not yet transmitted) node into the tree/list
    let nyt = huff.nodeList.as_mut_ptr().add(huff.blocNode as usize);
    huff.blocNode += 1;
    huff.tree = nyt;
    huff.lhead = nyt;
    huff.loc[NYT as usize] = nyt;
    (*huff.tree).symbol = NYT;
    (*huff.tree).weight = 0;
    (*huff.lhead).next = ptr::null_mut();
    (*huff.lhead).prev = ptr::null_mut();
    (*huff.tree).parent = ptr::null_mut();
    (*huff.tree).left = ptr::null_mut();
    (*huff.tree).right = ptr::null_mut();
    huff.loc[NYT as usize] = huff.tree;

    seq[0] = (size >> 8) as u8;
    seq[1] = (size & 0xff) as u8;

    let mut bloc: c_int = 16;

    for i in 0..size {
        let ch = *buffer.add(i as usize) as c_int;
        Huff_transmit(&mut huff, ch, seq.as_mut_ptr(), &mut bloc); // Transmit symbol
        Huff_addRef(&mut huff, ch as u8); // Do update
    }

    bloc += 8; // next byte

    (*mbuf).cursize = (bloc >> 3) + offset;
    ptr::copy_nonoverlapping(
        seq.as_ptr(),
        (*mbuf).data.add(offset as usize),
        (bloc >> 3) as usize,
    );
}

/// Raven `Huff_Init`.
///
/// Source: `oracle/codemp/qcommon/huffman.cpp:397-417`
pub unsafe fn Huff_Init(huff: *mut huffman_t) {
    (*huff).compressor = core::mem::zeroed();
    (*huff).decompressor = core::mem::zeroed();

    // Initialize the tree & list with the NYT node
    let d = &mut (*huff).decompressor;
    let dnyt = d.nodeList.as_mut_ptr().add(d.blocNode as usize);
    d.blocNode += 1;
    d.tree = dnyt;
    d.lhead = dnyt;
    d.ltail = dnyt;
    d.loc[NYT as usize] = dnyt;
    (*d.tree).symbol = NYT;
    (*d.tree).weight = 0;
    (*d.lhead).next = ptr::null_mut();
    (*d.lhead).prev = ptr::null_mut();
    (*d.tree).parent = ptr::null_mut();
    (*d.tree).left = ptr::null_mut();
    (*d.tree).right = ptr::null_mut();

    // Add the NYT (not yet transmitted) node into the tree/list
    let c = &mut (*huff).compressor;
    let cnyt = c.nodeList.as_mut_ptr().add(c.blocNode as usize);
    c.blocNode += 1;
    c.tree = cnyt;
    c.lhead = cnyt;
    c.loc[NYT as usize] = cnyt;
    (*c.tree).symbol = NYT;
    (*c.tree).weight = 0;
    (*c.lhead).next = ptr::null_mut();
    (*c.lhead).prev = ptr::null_mut();
    (*c.tree).parent = ptr::null_mut();
    (*c.tree).left = ptr::null_mut();
    (*c.tree).right = ptr::null_mut();
    c.loc[NYT as usize] = c.tree;
}
