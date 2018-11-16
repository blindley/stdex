// fn continuous_copy(sources: &[&[u8]], dests: &[&mut [u8]]) {
//     let mut source_arr = 0;
//     let mut dest_arr = 0;
//     let mut source_element = 0;
//     let mut dest_element = 0;
//     loop {
//         if source_arr == sources.len() || dest_arr == dests.len() {
//             break;
//         }

//         let source = sources[source_arr];
//         let dest = dests[dest_arr];
//         loop {
//             if source_element == source.len() {
//                 source_element = 0;
//                 source_arr += 1;
//                 break;
//             }

//             if dest_element == dest.len() {
//                 dest_element = 0;
//                 dest_arr += 1;
//                 break;
//             }

//             dest[dest_element] = source[source_element];
//             dest_element += 1;
//             source_element += 1;
//         }
//     }
// }

pub struct RingBuffer {
    data: Vec<u8>,
    write: usize,
}

impl RingBuffer {
    pub fn new(capacity: usize) -> RingBuffer {
        let mut data = Vec::with_capacity(capacity);
        unsafe { data.set_len(capacity); }
        RingBuffer { data, write: 0, }
    }

    // pub fn parts(&self) -> (&[u8], &[u8]) {
    //     let first = &self.data[self.write..];
    //     let second = &self.data[..self.write];
    //     (first, second)
    // }

    pub fn self_copy(&mut self, distance: usize, len: usize)
    -> Result<(), RingBufferError> {
        if distance > self.data.len() {
            return Err(RingBufferError::Underflow(
                RingBufferUnderflowError {
                    max: self.data.len(),
                    read_len: distance,
                }
            ));
        }

        let mut write = self.write;
        let mut read = (write + self.data.len() - distance) % self.data.len();

        let can_write = self.data.len() - write;
        let can_read = self.data.len() - read;
        let len1 = std::cmp::min(can_write, can_read);
        let len1 = std::cmp::min(len, len1);

        for _ in 0..len1 {
            self.data[write] = self.data[read];
            read += 1;
            write += 1;
        }

        let len = len - len1;
        read %= self.data.len();
        write %= self.data.len();

        let can_write = self.data.len() - write;
        let can_read = self.data.len() - read;
        let len1 = std::cmp::min(can_write, can_read);
        let len1 = std::cmp::min(len, len1);

        for _ in 0..len1 {
            self.data[write] = self.data[read];
            read += 1;
            write += 1;
        }

        self.write = write % self.data.len();

        Ok(())
    }

    pub fn copy_out(&self, buf: &mut [u8], back_offset: usize) {
        let back = back_offset % self.data.len();
        let mut read_start = (self.write + self.data.len() - back) % self.data.len();
        let mut write_start = 0;
        while write_start < buf.len() {
            let to_write = std::cmp::min(self.data.len() - read_start, buf.len() - write_start);
            let write_end = write_start + to_write;
            let read_end = read_start + to_write;
            buf[write_start..write_end].copy_from_slice(&self.data[read_start..read_end]);
            read_start = 0;
            write_start += to_write;
        }
    }
}

impl std::io::Write for RingBuffer {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let total_to_write = buf.len();

        let mut write_end = self.write + total_to_write;
        let mut read_start = 0;
        if write_end > self.data.len() {
            let to_write = self.data.len() - self.write;
            let read_end = read_start + to_write;
            self.data[self.write..].copy_from_slice(&buf[read_start..read_end]);
            read_start = read_end;
            self.write = 0;
            write_end = total_to_write - to_write;
        }

        let to_write = write_end - self.write;
        let read_end = read_start + to_write;
        self.data[self.write..write_end].copy_from_slice(&buf[read_start..read_end]);

        self.write = write_end;

        Ok(total_to_write)
    }

    fn flush(&mut self) -> std::io::Result<()> { Ok(()) }
}

#[derive(Debug, Clone, Copy)]
pub enum RingBufferError {
    Underflow(RingBufferUnderflowError),
}

impl std::fmt::Display for RingBufferError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            RingBufferError::Underflow(e) => write!(f, "{}", e),
        }
    }
}

impl std::error::Error for RingBufferError {}

impl From<RingBufferError> for std::io::Error {
    fn from(e: RingBufferError) -> Self {
        std::io::Error::new(std::io::ErrorKind::Other, e)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct RingBufferUnderflowError {
    pub max: usize,
    pub read_len: usize,
}

impl std::fmt::Display for RingBufferUnderflowError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

mod tests {
}
