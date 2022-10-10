use std::collections::VecDeque;

pub struct History<T> {
    pub history: VecDeque<T>,
    pub index: usize,
    pub max: usize,
}

impl<T> History<T> {
    pub fn new(max: usize) -> Self {
        Self {
            history: VecDeque::new(),
            index: 0,
            max,
        }
    }

    pub fn len(&self) -> usize {
        self.history.len()
    }

    pub fn add(&mut self, s: T) {
        self.history.push_back(s);
        if self.len() > self.max {
            self.history.pop_front();
        }
        self.index = self.len();
    }

    pub fn iter(&self) -> std::collections::vec_deque::Iter<T> {
        self.history.iter()
    }

    pub fn clear(&mut self) {
        self.history.clear();
        self.index = 0;
    }
}

impl<T: Clone> History<T> {
    pub fn prev(&mut self) -> Option<T> {
        if self.index == 0 {
            return None;
        }
        self.index -= 1;
        Some(self.history[self.index].clone())
    }

    pub fn next(&mut self) -> Option<T> {
        if self.index == self.history.len() {
            return None;
        }
        let out = Some(self.history[self.index].clone());
        if self.index < self.history.len() - 1 {
            self.index += 1;
        }
        out
    }

}

impl<T> std::ops::Index<usize> for History<T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        &self.history[index]
    }
}

impl<T> std::ops::IndexMut<usize> for History<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.history[index]
    }
}
