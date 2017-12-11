use std::collections::VecDeque;
use std::iter::Iterator;

pub struct MultiPeekable<I>
   where I: Iterator
{
   seen: VecDeque<I::Item>,
   iter: I,
   exhausted: bool,
}

impl <I> MultiPeekable<I>
   where I: Iterator
{
   pub fn new(iter: I)
      -> Self
   {
      MultiPeekable{iter: iter, seen: VecDeque::new(), exhausted: false}
   }

   pub fn peek(&mut self)
      -> Option<&I::Item>
   {
      self.peek_at(0)
   }

   pub fn peek_at(&mut self, n: usize)
      -> Option<&I::Item>
   {
      if n < self.seen.len()
      {
         self.seen.get(n)
      }
      else
      {
         // technically, an interator is not required to return None
         // if next is called again after None has been returned, so
         // record if end has been reached
         while !self.exhausted && n >= self.seen.len()
         {
            match self.iter.next()
            {
               Some(next) => self.seen.push_back(next),
               None => self.exhausted = true,
            }
         }
         if self.exhausted { None } else { self.seen.get(n) }
      }
   }

   fn get_next(&mut self)
      -> Option<I::Item>
   {
      if self.seen.len() > 0
      {
         self.seen.pop_front()
      }
      else if self.exhausted
      {
         // technically, an interator is not required to return None
         // if next is called again after None has been returned, so
         // record if end has been reached
         None
      }
      else
      {
         let result = self.iter.next();
         self.exhausted = result.is_none();

         result
      }
   }
}

impl <I> Iterator for MultiPeekable<I>
   where I: Iterator
{
   type Item = I::Item;

   fn next(&mut self)
      -> Option<I::Item>
   {
      self.get_next()
   }
}

#[cfg(test)]
mod test
{
   use super::MultiPeekable;
   #[test]
   fn test_peek()
   {
      let mut iter = MultiPeekable::new(1..6);
      assert_eq!(2, *iter.peek_at(1).unwrap());
      assert_eq!(1, *iter.peek().unwrap());
      assert_eq!(1, *iter.peek().unwrap());
      assert_eq!(2, *iter.peek_at(1).unwrap());
      assert_eq!(None, iter.peek_at(20));
      assert_eq!(1, *iter.peek().unwrap());
      assert_eq!(1, iter.next().unwrap());

      assert_eq!(3, *iter.peek_at(1).unwrap());
      assert_eq!(2, *iter.peek().unwrap());
      assert_eq!(2, *iter.peek().unwrap());
      assert_eq!(3, *iter.peek_at(1).unwrap());
      assert_eq!(3, *iter.peek_at(1).unwrap());
      assert_eq!(2, *iter.peek().unwrap());
      assert_eq!(2, iter.next().unwrap());

      assert_eq!(4, *iter.peek_at(1).unwrap());
      assert_eq!(3, *iter.peek().unwrap());
      assert_eq!(3, *iter.peek().unwrap());
      assert_eq!(4, *iter.peek_at(1).unwrap());
      assert_eq!(4, *iter.peek_at(1).unwrap());
      assert_eq!(3, *iter.peek().unwrap());
      assert_eq!(3, iter.next().unwrap());

      assert_eq!(5, *iter.peek_at(1).unwrap());
      assert_eq!(4, *iter.peek().unwrap());
      assert_eq!(4, *iter.peek().unwrap());
      assert_eq!(5, *iter.peek_at(1).unwrap());
      assert_eq!(5, *iter.peek_at(1).unwrap());
      assert_eq!(4, *iter.peek().unwrap());
      assert_eq!(4, iter.next().unwrap());

      assert_eq!(None, iter.peek_at(1));
      assert_eq!(5, *iter.peek().unwrap());
      assert_eq!(5, *iter.peek().unwrap());
      assert_eq!(None, iter.peek_at(1));
      assert_eq!(None, iter.peek_at(1));
      assert_eq!(5, *iter.peek().unwrap());
      assert_eq!(5, iter.next().unwrap());

      assert_eq!(None, iter.peek_at(1));
      assert_eq!(None, iter.peek());
      assert_eq!(None, iter.peek());
      assert_eq!(None, iter.peek_at(1));
      assert_eq!(None, iter.peek_at(1));
      assert_eq!(None, iter.peek());
      assert_eq!(None, iter.next());

      assert_eq!(None, iter.peek_at(1));
      assert_eq!(None, iter.peek());
      assert_eq!(None, iter.peek());
      assert_eq!(None, iter.peek_at(1));
      assert_eq!(None, iter.peek_at(1));
      assert_eq!(None, iter.peek());
      assert_eq!(None, iter.next());
   }
}
