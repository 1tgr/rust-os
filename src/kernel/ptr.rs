pub fn bytes_between<T>(ptr1: *const T, ptr2: *const T) -> usize {
    if ptr2 > ptr1 {
        ptr2 as usize - ptr1 as usize
    } else {
        ptr1 as usize - ptr2 as usize
    }
}

pub trait Align: Sized {
    fn down(value: Self, round: usize) -> Self;
    fn up(value: Self, round: usize) -> Self;
    fn range(base: Self, len: usize, round: usize) -> (Self, usize);
    fn is_aligned(value: Self, round: usize) -> bool;
}

impl Align for usize {
    fn down(value: usize, round: usize) -> usize {
        value & !(round - 1)
    }

    fn up(value: usize, round: usize) -> usize {
        Align::down(value + round - 1, round)
    }

    fn range(base: usize, len: usize, round: usize) -> (usize, usize) {
        let end = Align::up(base + len, round);
        let base = Align::down(base, round);
        (base, end - base)
    }

    fn is_aligned(value: Self, round: usize) -> bool {
        value & (round - 1) == 0
    }
}

impl<T> Align for *const T {
    fn down(value: *const T, round: usize) -> *const T {
        Align::down(value as usize, round) as *const T
    }

    fn up(value: *const T, round: usize) -> *const T {
        Align::up(value as usize, round) as *const T
    }

    fn range(base: *const T, len: usize, round: usize) -> (*const T, usize) {
        let (base, len) = Align::range(base as usize, len, round);
        (base as *const T, len)
    }

    fn is_aligned(value: *const T, round: usize) -> bool {
        Align::is_aligned(value as usize, round)
    }
}

impl<T> Align for *mut T {
    fn down(value: *mut T, round: usize) -> *mut T {
        Align::down(value as usize, round) as *mut T
    }

    fn up(value: *mut T, round: usize) -> *mut T {
        Align::up(value as usize, round) as *mut T
    }

    fn range(base: *mut T, len: usize, round: usize) -> (*mut T, usize) {
        let (base, len) = Align::range(base as usize, len, round);
        (base as *mut T, len)
    }

    fn is_aligned(value: *mut T, round: usize) -> bool {
        Align::is_aligned(value as usize, round)
    }
}

#[cfg(feature = "test")]
pub mod test {
    use super::*;

    test! {
        fn can_align_down() {
            let ptr = 0x1234 as *const u8;
            let expected_ptr = 0x1000 as *const u8;
            assert_eq!(expected_ptr, Align::down(ptr, 0x1000));
            assert_eq!(expected_ptr, Align::down(expected_ptr, 0x1000));
        }

        fn can_align_up() {
            let ptr = 0x1234 as *const u8;
            let expected_ptr = 0x2000 as *const u8;
            assert_eq!(expected_ptr, Align::up(ptr, 0x1000));
            assert_eq!(expected_ptr, Align::up(expected_ptr, 0x1000));
        }

        fn can_align_range() {
            let base = 0x1234 as *const u8;
            let len = 0x5678;
            let expected_base = 0x1000 as *const u8;
            let expected_len = 0x6000;
            assert_eq!((expected_base, expected_len), Align::range(base, len, 0x1000));
            assert_eq!((expected_base, expected_len), Align::range(expected_base, expected_len, 0x1000));
        }
    }
}
