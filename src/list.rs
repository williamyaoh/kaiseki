//! Doubly-linked list structure, supporting efficient pushes and appends
//! to either end.
//!
//! The only reason this module exists is because `std::collections::LinkedList`
//! doesn't have an `append_front()` method, for some ungodly reason.

use std::marker::PhantomData;
use std::mem;

use std::iter::FromIterator;

/// A doubly-linked list with owned nodes.
/// `List` allows pushing and popping elements at either end in constant time.
pub struct List<T> {
  front: Option<*mut Node<T>>,
  back: Option<*mut Node<T>>,
  len: usize,
  marker: PhantomData<Box<Node<T>>>
}

pub struct Iter<'a, T: 'a> {
  front: Option<*mut Node<T>>,
  back: Option<*mut Node<T>>,
  len: usize,
  marker: PhantomData<&'a Node<T>>
}

pub struct IntoIter<T> {
  list: List<T>
}
 
struct Node<T> {
  to_f: Option<*mut Node<T>>,
  to_b: Option<*mut Node<T>>,
  data: T
}

impl<T> List<T> {
  /// Create an empty `List`.
  pub fn new() -> Self {
    List { front: None, back: None, len: 0, marker: PhantomData }
  }
  
  /// Return the length of the list.
  /// Runs in O(1) space and O(1) time.
  ///
  /// # Examples
  ///
  /// ```
  /// use kaiseki::list::List;
  ///
  /// let mut dl = List::new();
  ///
  /// assert_eq!(dl.len(), 0);
  ///
  /// dl.push_back(2);
  /// assert_eq!(dl.len(), 1);
  /// dl.push_back(4);
  /// assert_eq!(dl.len(), 2);
  /// dl.push_front(1);
  /// assert_eq!(dl.len(), 3);
  /// ```
  pub fn len(&self) -> usize {
    self.len
  }

  /// Check if the list is empty.
  /// Runs in O(1) space and O(1) time.
  ///
  /// # Examples
  ///
  /// ```
  /// use kaiseki::list::List;
  ///
  /// let mut dl = List::new();

  /// assert!(dl.is_empty());

  /// dl.push_back(4);
  /// assert!(!dl.is_empty());
  /// ```
  pub fn is_empty(&self) -> bool {
    self.len == 0
  }

  /// Place `element` *before* all elements in the list.
  ///
  /// Runs in O(1) space and O(1) time.
  ///
  /// # Examples
  ///
  /// ```
  /// use kaiseki::list::List;
  /// use std::iter::IntoIterator;
  ///
  /// let mut dl = List::new();
  ///
  /// dl.push_front(4);
  /// dl.push_front(3);
  /// dl.push_front(2);
  /// dl.push_front(1);
  ///
  /// let numbers: [u32; 4] = [1, 2, 3, 4];
  /// let collected: Vec<u32> = dl.into_iter().collect();
  /// assert_eq!(&numbers as &[u32], &collected as &[u32]);
  /// ```
  pub fn push_front(&mut self, element: T) {
    let node = Box::new(Node { to_f: None, to_b: self.front, data: element });
    let node_ptr = Some(Box::into_raw(node));

    match self.front {
      None => self.back = node_ptr,
      Some(front) => unsafe {
        (*front).to_f = node_ptr;
      }
    };

    self.front = node_ptr;
    self.len += 1;
  }

  /// Place `element` *after* all elements in the list.
  ///
  /// Runs in O(1) space and O(1) time.
  ///
  /// # Examples
  ///
  /// ```
  /// use kaiseki::list::List;
  /// use std::iter::IntoIterator;
  ///
  /// let mut dl = List::new();
  ///
  /// dl.push_back(1);
  /// dl.push_back(2);
  /// dl.push_back(3);
  /// dl.push_back(4);
  ///
  /// let numbers: [u32; 4] = [1, 2, 3, 4];
  /// let collected: Vec<u32> = dl.into_iter().collect();
  /// assert_eq!(&numbers as &[u32], &collected as &[u32]);
  /// ```
  pub fn push_back(&mut self, element: T) {
    let node = Box::new(Node { to_f: self.back, to_b: None, data: element });
    let node_ptr = Some(Box::into_raw(node));

    match self.back {
      None => self.front = node_ptr,
      Some(back) => unsafe {
        (*back).to_b = node_ptr;
      }
    };

    self.back = node_ptr;
    self.len += 1;
  }

  /// Remove the first element in the list and return it, if there is one.
  ///
  /// Runs in O(1) space and O(1) time.
  pub fn pop_front(&mut self) -> Option<T> {
    match self.front {
      None => None,
      Some(node) => unsafe {
        let node = Box::from_raw(node);

        match (*node).to_b {
          None => self.back = None,
          Some(next) => (*next).to_f = None
        };

        self.len -= 1;
        self.front = (*node).to_b;

        Some((*node).data)
      }
    }
  }

  /// Peek at the first element in the list without removing it, if there is one.
  ///
  /// Runs in O(1) space and O(1) time.
  pub fn front(&self) -> Option<&T> {
    match self.front {
      None => None,
      Some(node) => unsafe {
        Some(&(*node).data)
      }
    }
  }

  /// Peek at the last element in the list without removing it, if there is one.
  ///
  /// Runs in O(1) space and O(1) time.
  pub fn back(&self) -> Option<&T> {
    match self.back {
      None => None,
      Some(node) => unsafe {
        Some(&(*node).data)
      }
    }
  }

  /// Remove the last element in the list and return it, if there is one.
  ///
  /// Runs in O(1) space and O(1) time.
  pub fn pop_back(&mut self) -> Option<T> {
    match self.back {
      None => None,
      Some(node) => unsafe {
        let node = Box::from_raw(node);

        match (*node).to_f {
          None => self.front = None,
          Some(next) => (*next).to_b = None
        };

        self.len -= 1;
        self.back = (*node).to_f;

        Some((*node).data)
      }
    }
  }

  /// Place all elements in `other` *before* all elements in the list.
  /// Reuses the nodes in `other`, placing them into the list. After the operation,
  /// `other` becomes empty.
  ///
  /// Runs in O(1) space and O(1) time.
  ///
  /// # Examples
  ///
  /// ```
  /// use kaiseki::list::List;
  /// use std::iter::IntoIterator;
  ///
  /// let mut dl1 = List::new();
  /// let mut dl2 = List::new();
  ///
  /// dl1.push_back(3);
  /// dl1.push_back(4);
  /// dl2.push_back(1);
  /// dl2.push_back(2);
  ///
  /// dl1.append_front(&mut dl2);
  ///
  /// let numbers: [u32; 4] = [1, 2, 3, 4];
  /// let collected: Vec<u32> = dl1.into_iter().collect();
  ///
  /// assert_eq!(&numbers as &[u32], &collected as &[u32]);
  /// assert!(dl2.is_empty());
  /// ```
  pub fn append_front(&mut self, other: &mut Self) {
    match (self.front, other.back) {
      (_, None) => (),
      (None, _) => mem::swap(self, other),
      (Some(our_front), Some(their_back)) => unsafe {
        (*our_front).to_f = Some(their_back);
        (*their_back).to_b = Some(our_front);

        self.front = other.front;
        self.len += other.len;
        
        other.front = None;
        other.back = None;
        other.len = 0;
      }
    };
  }

  /// Place all elements in `other` *after* all elements in the list.
  /// Reuses the nodes in `other`, placing them into the list. After the operation,
  /// `other` becomes empty.
  ///
  /// Runs in O(1) space and O(1) time.
  ///
  /// # Examples
  ///
  /// ```
  /// use kaiseki::list::List;
  /// use std::iter::IntoIterator;
  ///
  /// let mut dl1 = List::new();
  /// let mut dl2 = List::new();
  ///
  /// dl1.push_back(1);
  /// dl1.push_back(2);
  /// dl2.push_back(3);
  /// dl2.push_back(4);
  ///
  /// dl1.append_back(&mut dl2);
  ///
  /// let numbers: [u32; 4] = [1, 2, 3, 4];
  /// let collected: Vec<u32> = dl1.into_iter().collect();
  ///
  /// assert_eq!(&numbers as &[u32], &collected as &[u32]);
  /// assert!(dl2.is_empty());
  /// ```
  pub fn append_back(&mut self, other: &mut Self) {
    match (self.back, other.front) {
      (_, None) => (),
      (None, _) => mem::swap(self, other),
      (Some(our_back), Some(their_front)) => unsafe {
        (*our_back).to_b = Some(their_front);
        (*their_front).to_f = Some(our_back);

        self.back = other.back;
        self.len += other.len;

        other.front = None;
        other.back = None;
        other.len = 0;
      }
    };
  }

  pub fn iter(&self) -> Iter<T> {
    Iter { 
      front: self.front,
      back: self.back,
      len: self.len,
      marker: PhantomData
    }
  }
}

impl<T> Drop for List<T> {
  fn drop(&mut self) {
    let mut here = self.front;

    while let Some(node) = here {
      unsafe {
        let node = Box::from_raw(node);
        here = node.to_b;
      }
    }
  }
}

impl<T> IntoIterator for List<T> {
  type Item = T;
  type IntoIter = IntoIter<T>;

  fn into_iter(self) -> IntoIter<T> {
    IntoIter {
      list: self
    }
  }
}

impl<'a, T> Iterator for Iter<'a, T> {
  type Item = &'a T;

  fn next(&mut self) -> Option<&'a T> {
    unsafe {
      if self.len == 0 { return None; }

      let node = self.front
        .expect("invariant violated: front is None");

      self.len -= 1;
      self.front = (*node).to_b;

      Some(&(*node).data)
    }
  }
}

impl<'a, T> ExactSizeIterator for Iter<'a, T> {
  fn len(&self) -> usize {
    self.len
  }
}

impl<'a, T> DoubleEndedIterator for Iter<'a, T> {
  fn next_back(&mut self) -> Option<&'a T> {
    unsafe {
      if self.len == 0 { return None; }

      let node = self.back
        .expect("invariant violated: back is None");

      self.len -= 1;
      self.back = (*node).to_f;

      Some(&(*node).data)
    }
  }
}

impl<T> Iterator for IntoIter<T> {
  type Item = T;

  fn next(&mut self) -> Option<T> {
    self.list.pop_front()
  }
}

impl<T> ExactSizeIterator for IntoIter<T> {
  fn len(&self) -> usize {
    self.list.len
  }
}

impl<T> DoubleEndedIterator for IntoIter<T> {
  fn next_back(&mut self) -> Option<T> {
    self.list.pop_back()
  }
}

impl<A> FromIterator<A> for List<A>
{
  /// # Examples
  ///
  /// Using it directly:
  ///
  /// ```
  /// use kaiseki::list::List;
  /// use std::iter::{IntoIterator, FromIterator};
  ///
  /// let numbers: Vec<u32> = vec![1, 2, 3, 4];
  /// let list = List::from_iter(numbers);
  ///
  /// assert_eq!(list.len(), 4);
  ///
  /// let mut iter = list.into_iter();
  /// assert_eq!(iter.next(), Some(1));
  /// assert_eq!(iter.next(), Some(2));
  /// assert_eq!(iter.next(), Some(3));
  /// assert_eq!(iter.next(), Some(4));
  /// assert_eq!(iter.next(), None);
  /// ```
  ///
  /// Through `collect()`:
  ///
  /// ```
  /// use kaiseki::list::List;
  /// use std::iter::IntoIterator;
  ///
  /// let numbers: Vec<u32> = vec![1, 2, 3, 4];
  /// let list: List<u32> = numbers.into_iter().collect();
  ///
  /// assert_eq!(list.len(), 4);
  ///
  /// let mut iter = list.into_iter();
  /// assert_eq!(iter.next(), Some(1));
  /// assert_eq!(iter.next(), Some(2));
  /// assert_eq!(iter.next(), Some(3));
  /// assert_eq!(iter.next(), Some(4));
  /// assert_eq!(iter.next(), None);
  /// ```
  fn from_iter<I>(iter: I) -> Self where
    I: IntoIterator<Item=A>
  {
    let mut result = List::new();

    for element in iter {
      result.push_back(element);
    }
    
    result
  }
}
