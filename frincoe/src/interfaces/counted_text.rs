/**
Unauthorized text with a count and a name, used for tests.

The format of response is `recv({name}) {id} {text.len}: {text}\n`,
where `id` is the amount of messages sent, including current message,
and `text` is the text in the request.

Example:
```
# use frincoe::interfaces;
use interfaces::{CountedText, CountedTextProvider};

let mut msg = CountedTextProvider::new("fc");
assert_eq!(msg.send("hello"), "recv(fc) 1 5: hello\n");
assert_eq!(msg.send("hi"), "recv(fc) 2 2: hi\n");
```
*/
pub trait CountedText {
    /// Send the text, returning the response.
    fn send(&mut self, text: impl ToString) -> String;
}



/**
Provide an implement to [`CountedText`].

For detailed document, see [`CountedText`].
*/
#[derive(Debug, Clone)]
pub struct CountedTextProvider {
    name: String,
    id: u32,
}

impl CountedTextProvider {
    /// Create an object counting from 0
    pub fn new(name: impl ToString) -> Self {
        Self {
            name: name.to_string(),
            id: 0,
        }
    }
}

impl CountedText for CountedTextProvider {
    fn send(&mut self, text: impl ToString) -> String {
        self.id += 1;
        let text = text.to_string();
        format!("recv({}) {} {}: {}\n", self.name, self.id, text.len(), text)
    }
}
