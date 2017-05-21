## libimagapi

This library implements API components for imag, which then can be used to
implement scripting/language bindings.

For example the ownership/borrowing-model does not exist in other programming
languages.
Therefor, this library implements a cache object, so handles can be used to
operate on store objects (`StoreId`, `FileLockEntry`) and the like.

