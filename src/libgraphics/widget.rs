use cairo::cairo::Cairo;
use collections::vec_deque::VecDeque;
use std::sync::Arc;
use syscall::Result;
use types::Rect;

pub trait Widget {
    fn paint_on(&self, cr: &Cairo);
    fn move_to(&self, pos: Rect) -> Result<()>;
}

pub struct WidgetTree<T> {
    paint_needed: bool,
    by_zorder: VecDeque<Arc<T>>,
    focus: Option<Arc<T>>,
}

fn ref_eq<T>(a: &T, b: &T) -> bool {
    a as *const T == b as *const T
}

impl<T> WidgetTree<T> {
    pub fn new() -> Self {
        WidgetTree {
            paint_needed: true,
            by_zorder: VecDeque::new(),
            focus: None
        }
    }

    pub fn add(&mut self, widget: Arc<T>) {
        self.by_zorder.push_front(widget.clone());
        self.focus = Some(widget);
        self.paint_needed = true;
    }

    pub fn remove(&mut self, widget: &Arc<T>) {
        let index_opt =
            self.by_zorder
                .iter()
                .position(|w| ref_eq::<T>(w.as_ref(), widget.as_ref()));

        if let Some(index) = index_opt {
            self.by_zorder.remove(index);
        }

        let has_focus =
            self.focus
                .as_ref()
                .map_or(false, |old_focus| ref_eq::<T>(&*old_focus, &*widget));

        if has_focus {
            self.focus =
                index_opt
                    .and_then(|index| self.by_zorder.get(index))
                    .or_else(|| self.by_zorder.front())
                    .map(|w| (*w).clone());
        }

        self.paint_needed = true;
    }

    pub fn get_paint_needed(&mut self) -> bool {
        self.paint_needed
    }

    pub fn set_paint_needed(&mut self) {
        self.paint_needed = true;
    }

    pub fn get_focus(&self) -> Option<&Arc<T>> {
        self.focus.as_ref()
    }

    pub fn get_focus_mut(&mut self) -> Option<&mut Arc<T>> {
        self.focus.as_mut()
    }
}

impl<T: Widget> WidgetTree<T> {
    pub fn move_to(&mut self, widget: &Arc<T>, pos: Rect) -> Result<()> {
        widget.move_to(pos)?;
        self.paint_needed = true;
        Ok(())
    }

    pub fn paint_on(&mut self, cr: &Cairo) {
        cr.set_source_rgb(0.0, 0.0, 0.5);
        cr.paint();

        let mut i = self.by_zorder.iter();
        while let Some(widget) = i.next_back() {
            widget.paint_on(cr);
        }

        self.paint_needed = false;
    }
}

#[cfg(feature = "test")]
pub mod test {
    use super::*;

    pub trait PartialEqWith {
        type Inner;
        fn eq_with<F: FnOnce(&Self::Inner, &Self::Inner) -> bool>(&self, other: &Self, f: F) -> bool;
    }

    impl<T> PartialEqWith for Option<T> {
        type Inner = T;

        fn eq_with<F: FnOnce(&Self::Inner, &Self::Inner) -> bool>(&self, other: &Self, f: F) -> bool {
            match (self.as_ref(), other.as_ref()) {
                (Some(ref a), Some(ref b)) => f(a, b),
                (None, None) => true,
                _ => false
            }
        }
    }

    fn assert_focus_is<T>(tree: &WidgetTree<T>, widget: Option<&Arc<T>>) {
        assert!(tree.get_focus().eq_with(&widget, |a, b| {
            ref_eq::<T>(a.as_ref(), b.as_ref())
        }));
    }

    test! {
        fn add_changes_focus() {
            let one = Arc::new(1);
            let two = Arc::new(2);
            let mut tree = WidgetTree::new();
            assert_focus_is(&tree, None);
            tree.add(one.clone());
            assert_focus_is(&tree, Some(&one));
            tree.add(two.clone());
            assert_focus_is(&tree, Some(&two));
        }

        fn remove_one_does_not_change_focus() {
            let one = Arc::new(1);
            let two = Arc::new(2);
            let mut tree = WidgetTree::new();
            tree.add(one.clone());
            tree.add(two.clone());
            tree.remove(&one);
            assert_focus_is(&tree, Some(&two));
        }

        fn remove_two_changes_focus() {
            let one = Arc::new(1);
            let two = Arc::new(2);
            let mut tree = WidgetTree::new();
            tree.add(one.clone());
            tree.add(two.clone());
            tree.remove(&two);
            assert_focus_is(&tree, Some(&one));
        }

        fn remove_both_changes_focus() {
            let one = Arc::new(1);
            let two = Arc::new(2);
            let mut tree = WidgetTree::new();
            tree.add(one.clone());
            tree.add(two.clone());
            tree.remove(&two);
            tree.remove(&one);
            assert_focus_is(&tree, None);
        }
    }
}
