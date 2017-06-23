//! A stream combinator that buffers a number of elements before yielding them.
extern crate futures;

use std::collections::VecDeque;
use futures::{Stream, Async, Poll};

/// A stream combinator that buffers a number of elements before yielding them.
#[derive(Debug)]
#[must_use = "streams do nothing unless polled"]
pub struct After<S: Stream> {
	stream: S,
	amt: usize,
	items: VecDeque<S::Item>,
}

fn new<S: Stream>(stream: S, amt: usize) -> After<S> {
	After {
		stream,
		amt,
		items: VecDeque::with_capacity(amt),
	}
}

impl<S: Stream> Stream for After<S> {
	type Item = S::Item;
	type Error = S::Error;

	fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
		loop {
			match self.stream.poll() {
				Ok(Async::Ready(Some(item))) => {
					self.items.push_back(item);
					if self.items.len() > self.amt {
						let item = self.items.pop_front().expect("!self.items.is_empty(); qed");
						return Ok(Async::Ready(Some(item)))
					}
				},
				Ok(Async::Ready(None)) => return Ok(Async::Ready(None)),
				Ok(Async::NotReady) => return Ok(Async::NotReady),
				Err(err) => return Err(err),
			}
		}
	}
}

/// `AfterStream` adds `after` combinator to `Stream`.
pub trait AfterStream {
	fn after(self, amt: usize) -> After<Self> where Self: Stream + Sized;
}

impl<S> AfterStream for S where S: Stream {
	fn after(self, amt: usize) -> After<Self> {
		new(self, amt)
	}
}

#[cfg(test)]
mod tests {
	use futures::{Future, Stream, stream};
	use AfterStream;

	#[test]
	fn playpen() {
		let v: Vec<Result<u32, ()>> = vec![0, 1, 2, 3, 4, 5].into_iter().map(Ok).collect();
		let result = stream::iter(v)
			.after(4)
			.collect()
			.wait();
		assert_eq!(result, Ok(vec![0, 1]));
	}
}
