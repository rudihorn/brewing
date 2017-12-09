
pub struct CyclicBuffer<'a, T: 'a +  Copy> {
    pub data : &'a mut [T],
    pub ptr : usize,
    pub len : usize
}

impl<'a, T: 'a + Copy> CyclicBuffer<'a, T> {
    #[inline(always)]
    pub fn write(&mut self, dat : T) -> bool {
        if self.len >= self.data.len() {
            false
        } else {
            let i = (self.ptr + self.len) % self.data.len();
            self.data[i] = dat;
            self.len = self.len + 1;
            true
        }
    }

    #[inline(always)]
    pub fn read(&mut self) -> Option<T> {
        if self.empty() {
            None
        } else {
            let res = Some(self.data[self.ptr] as T);
            self.len = self.len - 1;
            self.ptr = (self.ptr + 1) % self.data.len();
            res
        }
    }

    #[inline(always)]
    pub fn empty(&self) -> bool {
        self.len == 0
    }
}