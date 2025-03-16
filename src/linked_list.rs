use std::{cell::RefCell, fmt::Debug, ops::Deref, rc::Rc};

#[derive(Debug)]
pub enum LLNode<T: Debug> {
    Node(T, Rc<RefCell<LLNode<T>>>),
    EOL,
}

pub struct LinkedList<T: Debug> {
    head: Rc<RefCell<LLNode<T>>>,
    current: Option<Rc<RefCell<LLNode<T>>>>,
}

impl<T: Debug> LinkedList<T> {
    pub fn display(&self) {
        let mut current = Rc::clone(&self.head);
        while let LLNode::Node(ref value, ref next) = current.clone().borrow().deref() {
            print!("{:?} -> ", value);
            current = Rc::clone(next);
        }
        println!("EOL");
    }

    pub fn new() -> LinkedList<T> {
        let head = Rc::new(RefCell::new(LLNode::EOL));
        LinkedList {
            head: head,
            current: None,
        }
    }

    pub fn append(&mut self, value: T) {
        match self.head.clone().borrow().deref() {
            LLNode::EOL => {
                self.head = Rc::new(RefCell::new(LLNode::Node(
                    value,
                    Rc::new(RefCell::new(LLNode::EOL)),
                )))
            }
            LLNode::Node(_, next) => {
                let mut last = Rc::clone(&next);

                while let LLNode::Node(_, next) = last.clone().borrow().deref() {
                    last = Rc::clone(next);
                }

                *last.borrow_mut() = LLNode::Node(value, Rc::new(RefCell::new(LLNode::EOL)));
            }
        };
    }

    pub fn push(&mut self, value: T) {
        match self.head.clone().borrow().deref() {
            LLNode::EOL => {
                self.head = Rc::new(RefCell::new(LLNode::Node(
                    value,
                    Rc::new(RefCell::new(LLNode::EOL)),
                )))
            }
            LLNode::Node(_, _) => {
                let old_head = Rc::clone(&self.head);
                self.head = Rc::new(RefCell::new(LLNode::Node(value, old_head)));
            }
        };
    }
}

impl<T: Debug + Copy> Iterator for LinkedList<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        match self.current.clone() {
            Some(ref current) => match current.deref().borrow().deref() {
                LLNode::Node(value, next) => {
                    self.current = Some(Rc::clone(&next));
                    return Some(*value);
                }
                LLNode::EOL => {
                    self.current = None;
                    return None;
                }
            },
            None => {
                match *self.head.borrow() {
                    LLNode::EOL => return None,
                    LLNode::Node(ref v, ref n) => {
                        self.current = Some(Rc::clone(n));
                        return Some(*v);
                    }
                };
            }
        };
    }
}

#[cfg(test)]
mod linkedlist_test {

    use std::ops::Deref;

    use super::{LLNode, LinkedList};

    #[test]
    fn linked_list_create() {
        let linked_list: LinkedList<i32> = LinkedList::new();
        match linked_list.head.deref().borrow().deref() {
            LLNode::Node(_, _) => {
                assert!(true);
            }
            LLNode::EOL => {}
        };
    }

    #[test]
    fn linked_list_mutation() {
        let mut linked_list = LinkedList::new();

        linked_list.display();

        linked_list.append(10);
        linked_list.append(12);
        linked_list.append(13);

        linked_list.push(0);
        linked_list.push(1);

        let mut iter = linked_list.into_iter();
        assert_eq!(iter.next(), Some(1));
        assert_eq!(iter.next(), Some(0));
        assert_eq!(iter.next(), Some(10));
        assert_eq!(iter.next(), Some(12));
        assert_eq!(iter.next(), Some(13));
    }
}
