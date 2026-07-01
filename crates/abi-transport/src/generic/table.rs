/// A typed engine-to-module function table.
///
/// These are the callbacks the executable provides through table ABIs such as
/// SP game's `GetGameAPI`.
pub trait FunctionTableImport {
    type Table;
}

/// A typed module-to-engine function table.
///
/// These are the callbacks the module returns through table ABIs such as SP
/// game's `GetGameAPI`.
pub trait FunctionTableExport {
    type Table;
}
