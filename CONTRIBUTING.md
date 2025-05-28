For abstract types, we generally use the following structure:

* `Name` is a trait that describes the type.
* `PyName` is a composite trait that implements `Name` for anything that implements `IntoPyObject`.
* `GenericName` is a generic struct that wraps PyObject and implements `PyName` for it.
