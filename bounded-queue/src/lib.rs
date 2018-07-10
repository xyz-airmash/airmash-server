//! Fast, zero-allocation, bounded queue implementation.

use std::mem;

#[derive(Clone, Debug)]
pub struct BoundedQueue<T> {
	front: usize,
	back: usize,
	vals: Box<[T]>
}

impl<T> BoundedQueue<T> {
	pub fn len(&self) -> usize {
		self.front - self.back
	}
	pub fn is_empty(&self) -> bool {
		self.front == self.back
	}
	pub fn is_full(&self) -> bool {
		self.len() == self.capacity()
	}
	pub fn capacity(&self) -> usize {
		self.vals.len()
	}

	fn last(&self) -> usize {
		self.back % self.len()
	}
	fn first(&self) -> usize {
		self.front % self.len()
	}

	pub fn push(&mut self, val: T) -> Option<T> {
		let val = mem::replace(&mut self.vals[self.first()], val);

		self.front += 1;

		if self.is_full() {
			self.back += 1;
			Some(val)
		}
		else {
			None
		}
	}
}

impl<T: Default> BoundedQueue<T> {
	pub fn pop(&mut self) -> Option<T> {
		if self.is_empty() { return None; }

		let last = self.last();
		self.back += 1;

		Some(mem::replace(&mut self.vals[last], T::default()))
	}
	pub fn peek(&self) -> Option<&T> {
		if self.is_empty() { return None; }

		Some(&self.vals[self.last()])
	}
}

impl<T: Clone + Default> BoundedQueue<T> {
	pub fn new(size: usize) -> Self {
		Self {
			front: 0,
			back: 0,
			vals: vec![T::default(); size].into_boxed_slice()
		}
	}
}