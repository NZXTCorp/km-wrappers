use crate::AsRawPtr;
use bitflags::bitflags;
use core::{marker::PhantomData, mem::size_of, ptr::null_mut};
use km_shared::strings::UnicodeString;
use km_sys::{
    HANDLE, OBJECT_ATTRIBUTES, OBJ_FORCE_ACCESS_CHECK, OBJ_KERNEL_HANDLE, OBJ_OPENIF,
    SECURITY_DESCRIPTOR, ULONG,
};

/// A strongly typed [`OBJECT_ATTRIBUTES`][msdn] structure.
///
/// [msdn]: https://docs.microsoft.com/en-us/windows/win32/api/ntdef/ns-ntdef-_object_attributes
#[repr(transparent)]
pub struct ObjectAttributes<'a, 'b>(
    OBJECT_ATTRIBUTES,
    PhantomData<&'a UnicodeString>,
    PhantomData<&'b SECURITY_DESCRIPTOR>,
);

impl<'a, 'b> ObjectAttributes<'a, 'b> {
    /// Creates a new object attributes structure.
    ///
    /// Port of the [`InitializeObjectAttributes` macro][macro] of the WDK.
    ///
    /// # Safety
    ///
    /// The caller must ensure that the `name`, `root_object_directory`, and `security_descriptor`
    /// parameters are valid.
    ///
    /// [macro]: https://docs.microsoft.com/en-us/windows/win32/api/ntdef/nf-ntdef-initializeobjectattributes
    pub unsafe fn initialize(
        name: &'a UnicodeString,
        flags: ObjectAttributesFlags,
        root_object_directory: Option<HANDLE>,
        security_descriptor: Option<&'b SECURITY_DESCRIPTOR>,
    ) -> Self {
        let object_attributes: OBJECT_ATTRIBUTES = OBJECT_ATTRIBUTES {
            Length: size_of::<OBJECT_ATTRIBUTES>() as u32,
            RootDirectory: root_object_directory.unwrap_or(null_mut()),
            Attributes: flags.bits(),
            // SAFETY: According to
            // https://docs.microsoft.com/en-us/windows/win32/api/ntdef/ns-ntdef-_object_attributes#remarks
            // object attributes are read-only, even though the pointers are not `*const` in its
            // definition.
            ObjectName: name as *const _ as *mut _,
            SecurityDescriptor: security_descriptor.as_raw_ptr() as *mut _,
            SecurityQualityOfService: null_mut(),
        };

        ObjectAttributes(object_attributes, PhantomData, PhantomData)
    }

    pub fn flags(&self) -> ObjectAttributesFlags {
        // SAFETY: Represented as true `ULONG` in the end, additional flags are ignored.
        unsafe { ObjectAttributesFlags::from_bits_unchecked(self.0.Attributes) }
    }
}

bitflags! {
    /// Object flags, see the [MSDN Documentation][msdn].
    ///
    /// [`OBJ_KERNEL_HANDLE`](ObjectAttributesFlags::OBJ_KERNEL_HANDLE) is set as part of this
    /// type's `Default` implementation.
    ///
    /// Note: This struct is incomplete; more flags can be added from that list as needed.
    ///
    /// [msdn]: https://docs.microsoft.com/en-us/windows/win32/api/ntdef/ns-ntdef-_object_attributes#members
    pub struct ObjectAttributesFlags: ULONG {
        /// The handle can only be accessed from kernel mode.
        const OBJ_KERNEL_HANDLE = OBJ_KERNEL_HANDLE;
        /// The routine that opens the handle should enforce all access checks for the object, even
        /// if the handle is being opened in kernel mode.
        const OBJ_FORCE_ACCESS_CHECK = OBJ_FORCE_ACCESS_CHECK;
        /// If this flag is specified, by using the object handle, to a routine that creates objects
        /// and if that object already exists, the routine should open that object. Otherwise, the
        /// routine creating the object returns an NTSTATUS code of
        /// [`crate::ntstatus::STATUS_OBJECT_NAME_COLLISION`].
        const OBJ_OPENIF = OBJ_OPENIF;
    }
}

impl Default for ObjectAttributesFlags {
    fn default() -> Self {
        ObjectAttributesFlags::OBJ_KERNEL_HANDLE
    }
}
