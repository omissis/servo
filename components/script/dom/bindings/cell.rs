/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

//! A shareable mutable container for the DOM.

use dom::bindings::trace::JSTraceable;
use js::jsapi::{JSTracer};

use util::task_state;
use util::task_state::{SCRIPT, IN_GC};

use std::cell::{RefCell, Ref, RefMut};

/// A mutable field in the DOM.
///
/// This extends the API of `core::cell::RefCell` to allow unsafe access in
/// certain situations, with dynamic checking in debug builds.
pub struct DOMRefCell<T> {
    value: RefCell<T>,
}

// Functionality specific to Servo's `DOMRefCell` type
// ===================================================

impl<T> DOMRefCell<T> {
    /// Return a reference to the contents.
    ///
    /// For use in the layout task only.
    pub unsafe fn borrow_for_layout<'a>(&'a self) -> &'a T {
        debug_assert!(task_state::get().is_layout());
        &*self.value.as_unsafe_cell().get()
    }

    /// Borrow the contents for the purpose of GC tracing.
    ///
    /// This succeeds even if the object is mutably borrowed,
    /// so you have to be careful in trace code!
    pub unsafe fn borrow_for_gc_trace<'a>(&'a self) -> &'a T {
        debug_assert!(task_state::get().contains(SCRIPT | IN_GC));
        &*self.value.as_unsafe_cell().get()
    }

    /// Borrow the contents for the purpose of script deallocation.
    ///
    pub unsafe fn borrow_for_script_deallocation<'a>(&'a self) -> &'a mut T {
        debug_assert!(task_state::get().contains(SCRIPT));
        &mut *self.value.as_unsafe_cell().get()
    }

    /// Is the cell mutably borrowed?
    ///
    /// For safety checks in debug builds only.
    pub fn is_mutably_borrowed(&self) -> bool {
        self.value.try_borrow().is_some()
    }

    /// Attempts to immutably borrow the wrapped value.
    ///
    /// The borrow lasts until the returned `Ref` exits scope. Multiple
    /// immutable borrows can be taken out at the same time.
    ///
    /// Returns `None` if the value is currently mutably borrowed.
    ///
    /// # Panics
    ///
    /// Panics if this is called off the script thread.
    pub fn try_borrow<'a>(&'a self) -> Option<Ref<'a, T>> {
        debug_assert!(task_state::get().is_script());
        self.value.try_borrow()
    }

    /// Mutably borrows the wrapped value.
    ///
    /// The borrow lasts until the returned `RefMut` exits scope. The value
    /// cannot be borrowed while this borrow is active.
    ///
    /// Returns `None` if the value is currently borrowed.
    ///
    /// # Panics
    ///
    /// Panics if this is called off the script thread.
    pub fn try_borrow_mut<'a>(&'a self) -> Option<RefMut<'a, T>> {
        debug_assert!(task_state::get().is_script());
        self.value.try_borrow_mut()
    }
}

impl<T: JSTraceable> JSTraceable for DOMRefCell<T> {
    fn trace(&self, trc: *mut JSTracer) {
        (*self).borrow().trace(trc)
    }
}

// Functionality duplicated with `core::cell::RefCell`
// ===================================================
impl<T> DOMRefCell<T> {
    /// Create a new `DOMRefCell` containing `value`.
    pub fn new(value: T) -> DOMRefCell<T> {
        DOMRefCell {
            value: RefCell::new(value),
        }
    }


    /// Immutably borrows the wrapped value.
    ///
    /// The borrow lasts until the returned `Ref` exits scope. Multiple
    /// immutable borrows can be taken out at the same time.
    ///
    /// # Panics
    ///
    /// Panics if this is called off the script thread.
    ///
    /// Panics if the value is currently mutably borrowed.
    pub fn borrow<'a>(&'a self) -> Ref<'a, T> {
        match self.try_borrow() {
            Some(ptr) => ptr,
            None => panic!("DOMRefCell<T> already mutably borrowed")
        }
    }

    /// Mutably borrows the wrapped value.
    ///
    /// The borrow lasts until the returned `RefMut` exits scope. The value
    /// cannot be borrowed while this borrow is active.
    ///
    /// # Panics
    ///
    /// Panics if this is called off the script thread.
    ///
    /// Panics if the value is currently borrowed.
    pub fn borrow_mut<'a>(&'a self) -> RefMut<'a, T> {
        match self.try_borrow_mut() {
            Some(ptr) => ptr,
            None => panic!("DOMRefCell<T> already borrowed")
        }
    }
}
