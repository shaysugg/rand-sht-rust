pub struct Stack<T> {
    inner: Vec<T>,
}

impl<T> Stack<T> {
    pub fn from(vec: Vec<T>) -> Stack<T> {
        Stack { inner: vec }
    }

    pub fn push(&mut self, value: T) {
        self.inner.push(value);
    }

    pub fn pop(&mut self) {
        self.inner.pop();
    }

    pub fn peak(&self) -> Option<&T> {
        self.inner.last()
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
}

#[cfg(test)]
mod stack_tests {
    use super::Stack;

    #[test]
    fn test_stack_push_pop() {
        let vec = vec![1, 2, 3, 4];
        let mut stack = Stack::from(vec);
        stack.push(5);
        stack.push(6);
        stack.pop();

        assert_eq!(stack.peak(), Some(&5))
    }
}
