# Code of Conduct

## Workflow

Don't commit on the main branch: it's for minor version advancing;
instead, commit on dev branch for corresponding version.

Commit messages should follow the format `verb component: description`,
where the component use `/` as path separator,
and can be omitted when it's hard to summarize >\_\<.
Possible verbs are:

- `feat`: A new feature is added,
  either by extending an existing component or adding a new component.
- `doc`: More document is added;
  however usually documents should be added alone with the feature itself.
- `fix`: An unexpected behavior is removed.
- `refa`: Refactors, e.g. some codes are extracted into a function,
  or a function is rewritten.
- `misc`: A lot things are updated... Avoid this if possible.

## Styling

Most of the time in normal code, `rustfmt` would handle the formation well;
any styling should first satisfy `rustfmt`.

The skeleton of a file should be like:
```rust
/*!
Inner docs for a file
*/

#![filewise_option]



use some_crate::Element;



// Documents should come with the item definition
mod submod1;
use self::submod1::Something;

/**
Something depending on [`Something`].
*/
pub struct Orz1 {
}



// Another section logically independent to the prior one
/**
Something [`SomeOther`] depends on.
*/
pub struct Orz2

mod submod2;
use self::submod2::SomeOther;



#[cfg(test)]
mod tests {
}
```

As it's shown in the skeleton,
large skips (3 empty lines) are used to divide 'section's,
and small skips (one empty line) are used as smaller separators.

For macros, keep them within a line if possible;
otherwise, since most of them adopt rust-like syntax,
format them as how `rustfmt` format normal rust code.

[//modeline]: vim: spell nofoldenable
