#[derive(Debug, Clone)]
enum DNode<T>
where
    T: Clone + std::fmt::Debug,
{
    First {
        value: T,
        next: *mut DNode<T>,
    },
    Body {
        value: T,
        next: *mut DNode<T>,
        prev: *mut DNode<T>,
    },
    Last {
        value: T,
        prev: *mut DNode<T>,
    },
}

impl<T: Clone + std::fmt::Debug> DNode<T> {
    fn as_body(&mut self, other: *mut Self) {
        let a = match &self {
            Self::First { value, next } => Self::Body {
                value: value.clone(),
                next: *next,
                prev: other,
            },
            Self::Body { value, next, prev } => Self::Body {
                value: value.clone(),
                next: *next,
                prev: *prev,
            },
            Self::Last { value, prev } => Self::Body {
                value: value.clone(),
                next: other,
                prev: *prev,
            },
        };
        *self = a;
    }

    const fn next(&self) -> Option<*mut Self> {
        match &self {
            Self::First { value: _, next }
            | Self::Body {
                value: _,
                next,
                prev: _,
            } => Some(*next),
            Self::Last { value: _, prev: _ } => None,
        }
    }

    const fn prev(&self) -> Option<*mut Self> {
        match &self {
            Self::First { value: _, next: _ } => None,
            Self::Body {
                value: _,
                next: _,
                prev,
            }
            | Self::Last { value: _, prev } => Some(*prev),
        }
    }
}

#[derive(Debug, Clone)]
enum DLPtr<T>
where
    T: Clone + std::fmt::Debug,
{
    Empty,
    Unit {
        value: T,
    },
    Multi {
        head: *mut DNode<T>,
        tail: *mut DNode<T>,
    },
}

#[derive(Debug, Clone)]
pub struct DLList<T>
where
    T: Clone + std::fmt::Debug,
{
    length: usize,
    ptr: DLPtr<T>,
}

impl<T: Clone + std::fmt::Debug> Drop for DLList<T> {
    fn drop(&mut self) {
        while self.remove_at(0).is_some() {}
    }
}

impl<T: Clone + std::fmt::Debug> DLList<T> {
    pub const fn new() -> Self {
        Self {
            length: 0,
            ptr: DLPtr::Empty,
        }
    }

    pub fn prepend(&mut self, item: T) {
        self.length += 1;
        match &self.ptr {
            DLPtr::Empty => self.ptr = DLPtr::Unit { value: item },

            DLPtr::Unit { value } => {
                let value = value.clone();
                let head = Box::into_raw(Box::new(DNode::First {
                    value: item,
                    next: std::ptr::null_mut(),
                }));
                let tail = Box::into_raw(Box::new(DNode::Last {
                    value,
                    prev: std::ptr::null_mut(),
                }));
                unsafe {
                    if let DNode::First { value: _, next } = &mut (*head) {
                        *next = tail;
                    }
                    if let DNode::Last { value: _, prev } = &mut (*tail) {
                        *prev = head;
                    }
                }
                self.ptr = DLPtr::Multi { head, tail }
            }

            DLPtr::Multi { head, tail } => {
                let new_head = DNode::First {
                    value: item,
                    next: *head,
                };
                let ptr_head = Box::into_raw(Box::new(new_head));
                unsafe {
                    (**head).as_body(ptr_head);
                }
                self.ptr = DLPtr::Multi {
                    head: ptr_head,
                    tail: *tail,
                }
            }
        }
    }

    pub fn append(&mut self, item: T) {
        self.length += 1;
        match &self.ptr {
            DLPtr::Empty => self.ptr = DLPtr::Unit { value: item },

            DLPtr::Unit { value } => {
                let value = value.clone();
                let head = Box::into_raw(Box::new(DNode::First {
                    value,
                    next: std::ptr::null_mut(),
                }));
                let tail = Box::into_raw(Box::new(DNode::Last {
                    value: item,
                    prev: std::ptr::null_mut(),
                }));
                unsafe {
                    if let DNode::First { value: _, next } = &mut (*head) {
                        *next = tail;
                    }
                    if let DNode::Last { value: _, prev } = &mut (*tail) {
                        *prev = head;
                    }
                }
                self.ptr = DLPtr::Multi { head, tail }
            }

            DLPtr::Multi { head, tail } => {
                let new_tail = DNode::Last {
                    value: item,
                    prev: *tail,
                };
                let ptr_tail = Box::into_raw(Box::new(new_tail));
                unsafe {
                    (**tail).as_body(ptr_tail);
                }
                self.ptr = DLPtr::Multi {
                    head: *head,
                    tail: ptr_tail,
                }
            }
        }
    }

    pub fn insert_at(&mut self, item: T, idx: usize) {
        let length = self.length;
        match &self.ptr {
            DLPtr::Empty => self.append(item),
            DLPtr::Unit { value: _ } => match idx {
                0 => self.prepend(item),
                1 => self.append(item),
                _ => (),
            },
            DLPtr::Multi { head, tail: _ } => match idx {
                0 => self.prepend(item),
                a if a < length => {
                    self.length += 1;
                    let mut curr = *head;
                    for _ in 0..a {
                        unsafe {
                            curr = (*curr).next().expect("Impossible");
                        }
                    }
                    let next = curr;
                    let prev = unsafe { (*curr).prev().expect("Impossible") };
                    let new_node = DNode::Body {
                        value: item,
                        next,
                        prev,
                    };
                    let new_ptr = Box::into_raw(Box::new(new_node));
                    unsafe {
                        match &mut (*next) {
                            DNode::First { value: _, next: _ } => (),
                            DNode::Body {
                                value: _,
                                next: _,
                                prev,
                            }
                            | DNode::Last { value: _, prev } => *prev = new_ptr,
                        }
                        match &mut (*prev) {
                            DNode::First { value: _, next }
                            | DNode::Body {
                                value: _,
                                next,
                                prev: _,
                            } => *next = new_ptr,
                            DNode::Last { value: _, prev: _ } => (),
                        }
                    }
                }
                a if a == length => self.append(item),
                _ => (),
            },
        }
    }

    pub fn remove_at(&mut self, idx: usize) -> Option<T> {
        let length = self.length;
        if idx >= length {
            None
        } else {
            match &self.ptr {
                DLPtr::Empty => None,
                DLPtr::Unit { value } => {
                    let value = value.clone();
                    self.ptr = DLPtr::Empty;
                    self.length -= 1;
                    Some(value)
                }
                DLPtr::Multi { head, tail } => {
                    self.length -= 1;
                    if length == 2 {
                        match idx {
                            0 => {
                                let (DNode::Body {
                                    value,
                                    next: _,
                                    prev: _,
                                }
                                | DNode::First { value, next: _ }
                                | DNode::Last { value, prev: _ }) =
                                    unsafe { *Box::from_raw(*head) };

                                self.ptr = DLPtr::Unit {
                                    value: match unsafe { *Box::from_raw(*tail) } {
                                        DNode::Body {
                                            value,
                                            next: _,
                                            prev: _,
                                        }
                                        | DNode::First { value, next: _ }
                                        | DNode::Last { value, prev: _ } => value,
                                    },
                                };

                                Some(value)
                            }
                            1 => {
                                let (DNode::Body {
                                    value,
                                    next: _,
                                    prev: _,
                                }
                                | DNode::First { value, next: _ }
                                | DNode::Last { value, prev: _ }) =
                                    unsafe { *Box::from_raw(*tail) };

                                self.ptr = DLPtr::Unit {
                                    value: match unsafe { *Box::from_raw(*head) } {
                                        DNode::Body {
                                            value,
                                            next: _,
                                            prev: _,
                                        }
                                        | DNode::First { value, next: _ }
                                        | DNode::Last { value, prev: _ } => value,
                                    },
                                };

                                Some(value)
                            }
                            _ => None,
                        }
                    } else {
                        let mut curr = *head;
                        for _ in 0..idx {
                            curr = unsafe { (*curr).next().expect("Impossible") };
                        }
                        unsafe {
                            match &*Box::from_raw(curr) {
                                DNode::First { value, next } => {
                                    if let DLPtr::Multi { head, tail: _ } = &mut self.ptr {
                                        *head = *next;
                                    }
                                    if let DNode::Body {
                                        value: body_value,
                                        next: body,
                                        prev: _,
                                    } = (**next).clone()
                                    {
                                        **next = DNode::First {
                                            value: body_value,
                                            next: body,
                                        }
                                    }
                                    Some(value.clone())
                                }
                                DNode::Body { value, next, prev } => {
                                    let next_clone = *next;
                                    let prev_clone = *prev;
                                    if let DNode::First { value: _, next }
                                    | DNode::Body {
                                        value: _,
                                        next,
                                        prev: _,
                                    } = &mut **prev
                                    {
                                        *next = next_clone;
                                    }
                                    if let DNode::Last { value: _, prev }
                                    | DNode::Body {
                                        value: _,
                                        next: _,
                                        prev,
                                    } = &mut **next
                                    {
                                        *prev = prev_clone;
                                    }
                                    Some(value.clone())
                                }
                                DNode::Last { value, prev } => {
                                    if let DLPtr::Multi { head: _, tail } = &mut self.ptr {
                                        *tail = *prev;
                                    }
                                    if let DNode::Body {
                                        value: body_value,
                                        next: _,
                                        prev: body,
                                    } = (**prev).clone()
                                    {
                                        **prev = DNode::Last {
                                            value: body_value,
                                            prev: body,
                                        }
                                    }
                                    Some(value.clone())
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
