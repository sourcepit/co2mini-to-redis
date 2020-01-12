use common_failures::prelude::*;
use hidapi_sys::hid_close;
use hidapi_sys::hid_device;
use hidapi_sys::hid_device_info;
use hidapi_sys::hid_enumerate;
use hidapi_sys::hid_exit;
use hidapi_sys::hid_free_enumeration;
use hidapi_sys::hid_init;
use hidapi_sys::hid_open;
use hidapi_sys::hid_open_path;
use hidapi_sys::hid_read;
use hidapi_sys::hid_send_feature_report;
use std::ffi::CStr;
use std::ffi::CString;
use std::ptr;
use widestring::U32CString;

pub fn init() -> Result<()> {
    unsafe {
        match hid_init() {
            0 => Ok(()),
            _ => Err(format_err!("Failed to initialize the HIDAPI library.")),
        }
    }
}

pub fn exit() -> Result<()> {
    unsafe {
        match hid_exit() {
            0 => Ok(()),
            _ => Err(format_err!("Failed to free the HIDAPI library.")),
        }
    }
}

pub struct Device {
    hid_device: *mut hid_device,
}

impl Device {
    pub fn open(vendor_id: u16, product_id: u16, serial_number: Option<String>) -> Result<Device> {
        unsafe {
            let serial_number = match serial_number {
                Some(v) => U32CString::from_str(v).unwrap().into_raw(),
                None => ptr::null(),
            };

            let hid_device = hid_open(vendor_id, product_id, serial_number);
            Device::new(hid_device)
        }
    }

    pub fn open_path(path: &str) -> Result<Device> {
        let path = CString::new(path)?;
        let hid_device: *mut hid_device = unsafe { hid_open_path(path.as_ptr()) };
        Device::new(hid_device)
    }

    fn new(hid_device: *mut hid_device) -> Result<Device> {
        match hid_device.is_null() {
            false => (),
            _ => return Err(format_err!("Failed to open HID device.")),
        };
        Ok(Device { hid_device })
    }

    pub fn send_feature_report(&self, data: &[u8]) -> Result<usize> {
        let res = unsafe { hid_send_feature_report(self.hid_device, data.as_ptr(), data.len()) };
        match res >= 0 {
            true => Ok(res as usize),
            _ => Err(format_err!("Failed to send feature report.")),
        }
    }

    pub fn read(&self, data: &mut [u8]) -> Result<usize> {
        let res = unsafe { hid_read(self.hid_device, data.as_mut_ptr(), data.len()) };
        match res >= 0 {
            true => Ok(res as usize),
            _ => Err(format_err!("Failed to read data.")),
        }
    }
}

impl Drop for Device {
    fn drop(&mut self) {
        unsafe {
            hid_close(self.hid_device);
        }
    }
}

#[derive(Clone, Debug)]
pub struct DeviceInfo {
    pub path: String,
    pub vendor_id: u16,
    pub product_id: u16,

    pub serial_number: Option<String>,
    pub release_number: u16,

    pub manufacturer_string: Option<String>,
    pub product_string: Option<String>,
    pub usage_page: u16,
    pub usage: u16,

    pub interface_number: i32,
}

/// An iterator over the available devices.
pub struct DeviceInfos {
    head: *mut hid_device_info,
    current: *mut hid_device_info,
}

impl DeviceInfos {
    pub fn new(vendor_id: Option<u16>, product_id: Option<u16>) -> DeviceInfos {
        let head = unsafe { hid_enumerate(vendor_id.unwrap_or(0), product_id.unwrap_or(0)) };
        DeviceInfos {
            head: head,
            current: head,
        }
    }
}

unsafe fn to_string(value: *const u32) -> String {
    U32CString::from_ptr_str(value).to_string().unwrap()
}

impl Iterator for DeviceInfos {
    type Item = DeviceInfo;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current.is_null() {
            return None;
        }

        let device_info = unsafe {
            let serial_number = (*self.current).serial_number;
            let serial_number = match serial_number.is_null() {
                false => serial_number.as_ref().and_then(|p| Some(to_string(p))),
                _ => None,
            };

            let manufacturer_string = (*self.current).manufacturer_string;
            let manufacturer_string = match manufacturer_string.is_null() {
                false => manufacturer_string
                    .as_ref()
                    .and_then(|p| Some(to_string(p))),
                _ => None,
            };

            let product_string = (*self.current).product_string;
            let product_string = match product_string.is_null() {
                false => product_string.as_ref().and_then(|p| Some(to_string(p))),
                _ => None,
            };

            let path = String::from(CStr::from_ptr((*self.current).path).to_str().unwrap());

            DeviceInfo {
                path: path,
                vendor_id: (*self.current).vendor_id,
                product_id: (*self.current).product_id,
                serial_number: serial_number,
                release_number: (*self.current).release_number,
                manufacturer_string: manufacturer_string,
                product_string: product_string,
                usage_page: (*self.current).usage_page,
                usage: (*self.current).usage,
                interface_number: (*self.current).interface_number,
            }
        };

        self.current = unsafe { (*self.current).next };

        Some(device_info)
    }
}

impl Drop for DeviceInfos {
    fn drop(&mut self) {
        println!("foo");
        unsafe {
            hid_free_enumeration(self.head);
        }
    }
}
