# Good to know

There are some things in the imag space that a user might want to know before
using imag.


## Tags

Tags are text values that can be added to _any_ entry in the imag store.
The tag has to satisfy some invariants to be valid:

1. it has to be all-lowercase
1. it has no spaces
1. all characters are alphanumeric

Any entry can have any number of tags. Spelling is not checked.

Finding all entries for a tag "foo" is `O(n)` where `n` is the number of entries
in the store.


## Categories

Categories are more restrictive, but also more powerful.
First of all, a category has to exist before an entry can be _linked_ to a
category. A category therefor is represented by an entry and if an entry has a
category, it is linked to said entry.

So, if you create only category with the name "foo", you cannot set the category
"bar" for some entry.

Because categories are linked, finding all entries for a category is trivial and
a `O(1)` operation.

