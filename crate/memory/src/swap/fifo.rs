use alloc::vec_deque::VecDeque;
use super::*;

struct FifoSwapManager {
    deque: VecDeque<VirtAddr>,
}

impl SwapManager for FifoSwapManager {
    fn tick(&mut self) {

    }

    fn push(&mut self, addr: usize) {
        self.deque.push_back(addr);
    }

    fn remove(&mut self, addr: usize) {
        let id = self.deque.iter()
            .position(|&x| x == addr)
            .expect("address not found");
        self.deque.remove(id);
    }

    fn pop(&mut self) -> Option<VirtAddr> {
        self.deque.pop_front()
    }
}

impl FifoSwapManager {
    fn new() -> Self {
        FifoSwapManager {
            deque: VecDeque::<VirtAddr>::new()
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use alloc::{arc::Arc, boxed::Box};
    use core::cell::RefCell;
    use page_table::mock_page_table::MockPageTable;

    enum MemOp {
        R(usize), W(usize)
    }

    #[test]
    fn test() {
        use self::MemOp::{R, W};
        let page_fault_count = Arc::new(RefCell::new(0usize));

        let mut pt = MockPageTable::new(4, Box::new({
            let page_fault_count1 = page_fault_count.clone();
            let mut fifo = FifoSwapManager::new();

            move |pt: &mut MockPageTable, addr: VirtAddr| {
                *page_fault_count1.borrow_mut() += 1;

                if !pt.map(addr) {  // is full?
                    pt.unmap(fifo.pop().unwrap());
                    pt.map(addr);
                }
                fifo.push(addr);
            }
        }));

        let op_seq = [
            R(1), R(2), R(3), R(4),
            W(3), W(1), W(4), W(2), W(5),
            W(2), W(1), W(2), W(3), W(4),
            W(5), R(1), W(1)];
        let pgfault_count = [
            1, 2, 3, 4,
            4, 4, 4, 4, 5,
            5, 6, 7, 8, 9,
            10, 11, 11];
        for (op, &count) in op_seq.iter().zip(pgfault_count.iter()) {
            match op {
                R(addr) => pt.read(*addr),
                W(addr) => pt.write(*addr),
            }
            assert_eq!(*(*page_fault_count).borrow(), count);
        }
    }
}