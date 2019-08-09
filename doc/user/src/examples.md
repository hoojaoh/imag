# Examples

imag commands can be chained by piping the output.
This can be used to create, tag and categorize an entry in one step.

```
imag log --to personal "Some personal note" | \
    imag tag add foobar | \
    imag category set somecategory
```

imag can be configured to use aliases for module commands, so combining a basi
alias `alias i=imag` with imag module aliaes, this can be condensed to:

```
i l --to personal "Some personal note" | \
    i t add foobar | \
    i c set somecategory
```

for example.

