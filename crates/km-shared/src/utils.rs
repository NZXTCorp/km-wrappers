use core::ptr;

pub trait AsRawPtr {
    type Pointee;
    fn as_raw_ptr(&self) -> *const Self::Pointee;
}

pub trait AsRawMutPtr {
    type Pointee;
    fn as_raw_mut_ptr(&mut self) -> *mut Self::Pointee;
}

impl<'a, T> AsRawPtr for Option<&'a T> {
    type Pointee = T;

    #[inline(always)]
    fn as_raw_ptr(&self) -> *const Self::Pointee {
        // this optimizes down to the the same assembly as
        // unsafe { ptr::read::<*const T>(self as *const _ as *const _) }
        self.map_or(ptr::null(), |x| x as *const _)
    }
}

impl<'a, T> AsRawPtr for Option<&'a mut T> {
    type Pointee = T;

    #[inline(always)]
    fn as_raw_ptr(&self) -> *const Self::Pointee {
        // SAFETY: `Option<&'a mut T>` has the same layout as a raw pointer
        unsafe { ptr::read::<*const T>(self as *const _ as *const _) }
    }
}

impl<'a, T> AsRawMutPtr for Option<&'a mut T> {
    type Pointee = T;

    #[inline(always)]
    fn as_raw_mut_ptr(&mut self) -> *mut Self::Pointee {
        // this optimizes down to the the same assembly as
        // unsafe { ptr::read::<*mut T>(self as *const _ as *const _) }
        self.as_mut().map_or(ptr::null_mut(), |x| (*x) as *mut _)
    }
}
