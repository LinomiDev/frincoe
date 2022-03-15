#![doc = concat! {r####"
Forward procedure calls between clients connected.

# Example: passive cable

```
"####,
include_str!("../examples/passive_cable_basic.rs"), r####"
```
"####}]

use std::ops::{Deref, DerefMut};
use std::vec::IntoIter;

use frincoe_rpc::Connection;



/**
Plug in servers and forward procedure calls between them, the passive version.

Since not all passive cables is an active cable at the same time,
this is the passive side of them.
*/
pub trait Cable<'a>: Connection {
    /// Type of clients owned by the cable.
    type Client: 'a;
    /// An iterator over the children
    type ChildIter: Iterator<Item = &'a mut Self::Client>;
    /// Returns an iterator over the children.
    fn iter_child(&'a mut self) -> Self::ChildIter;
    /// Add a connection to the cable.
    /// If there's any error, the connection is not added.
    fn add_connection(&mut self, addr: Self::Client) -> Result<(), Self::Error>;
}

/**
The results of a cabled procedure.

Due to some rust restrictions, this have to be a concrete type instead of a trait.

Since a cable method should assembly all its children's results,
and the signature of all these methods should be the same (well, constraintedly),
all cable methods should return a `Bundle`.
A better way is using a trait, so that some bundled values may have better optimization;
but the solution using traits requires features that rust don't implement currently.
*/
#[derive(Clone, Default, Debug)]
pub struct Bundle<T> {
    items: Vec<T>,
}

impl<T> Bundle<T> {
    /// Create an empty bundle.
    pub fn new() -> Self {
        Self { items: vec![] }
    }

    /// Create a bundle with a single value
    pub fn from_single(item: impl Into<T>) -> Self {
        Self {
            items: vec![item.into()],
        }
    }
}

impl<T, U: Into<T>> FromIterator<U> for Bundle<T> {
    fn from_iter<R: IntoIterator<Item = U>>(iter: R) -> Self {
        Self {
            items: iter.into_iter().map(Into::<T>::into).collect(),
        }
    }
}

impl<T> Extend<Bundle<T>> for Bundle<T> {
    fn extend<R: IntoIterator<Item = Bundle<T>>>(&mut self, iter: R) {
        self.items.extend(iter.into_iter().flat_map(|x| x.items.into_iter()))
    }

    fn extend_one(&mut self, item: Bundle<T>) {
        self.items.extend(item.items.into_iter())
    }

    fn extend_reserve(&mut self, additional: usize) {
        self.items.reserve(additional);
    }
}

impl<T> Extend<T> for Bundle<T> {
    fn extend<R: IntoIterator<Item = T>>(&mut self, iter: R) {
        self.items.extend(iter.into_iter());
    }

    fn extend_one(&mut self, item: T) {
        self.items.extend_one(item);
    }

    fn extend_reserve(&mut self, additional: usize) {
        self.items.reserve(additional);
    }
}

impl<T> IntoIterator for Bundle<T> {
    type IntoIter = IntoIter<T>;
    type Item = T;

    fn into_iter(self) -> Self::IntoIter {
        self.items.into_iter()
    }
}

impl<T> Deref for Bundle<T> {
    type Target = Vec<T>;

    fn deref(&self) -> &Self::Target {
        &self.items
    }
}

impl<T> DerefMut for Bundle<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.items
    }
}

impl<U, T: PartialEq<U>, const N: usize> PartialEq<[U; N]> for Bundle<T> {
    fn eq(&self, other: &[U; N]) -> bool {
        self.items == other
    }
}

impl<U, T: PartialEq<U>> PartialEq<[U]> for Bundle<T> {
    fn eq(&self, other: &[U]) -> bool {
        self.items == other
    }
}

impl<U, T: PartialEq<U>> PartialEq<&[U]> for Bundle<T> {
    fn eq(&self, other: &&[U]) -> bool {
        self.items == *other
    }
}

impl<U, T: PartialEq<U>> PartialEq<&mut [U]> for Bundle<T> {
    fn eq(&self, other: &&mut [U]) -> bool {
        self.items == *other
    }
}



/**
Cable containing a list of clients of the same type.
 */
#[derive(Clone, Default, Debug)]
pub struct ArrayCable<T> {
    child: Vec<T>,
}

impl<T> ArrayCable<T> {
    /// Create an empty ArrayCable
    pub fn new() -> Self {
        Self { child: vec![] }
    }
}

impl<T, U: Into<T>> FromIterator<U> for ArrayCable<T> {
    fn from_iter<R: IntoIterator<Item = U>>(iter: R) -> Self {
        Self {
            child: iter.into_iter().map(Into::<T>::into).collect(),
        }
    }
}

impl<T, U: Into<T>> Extend<U> for ArrayCable<T> {
    fn extend<R: IntoIterator<Item = U>>(&mut self, iter: R) {
        self.child.extend(iter.into_iter().map(|x| x.into()));
    }

    fn extend_one(&mut self, item: U) {
        self.child.extend_one(item.into());
    }

    fn extend_reserve(&mut self, additional: usize) {
        self.child.reserve(additional);
    }
}

impl<T> Connection for ArrayCable<T> {
    type Error = ();

    fn disconnect(&self) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl<'a, T: 'a> Cable<'a> for ArrayCable<T> {
    type ChildIter = core::slice::IterMut<'a, T>;
    type Client = T;

    fn iter_child(&'a mut self) -> Self::ChildIter {
        self.child.iter_mut()
    }

    fn add_connection(&mut self, addr: Self::Client) -> Result<(), Self::Error> {
        self.child.push(addr);
        Ok(())
    }
}



#[cfg(test)]
mod tests {}
