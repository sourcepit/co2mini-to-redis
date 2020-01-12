use common_failures::prelude::*;

use hidapi::Device;
use hidapi::DeviceInfos;

const VENDOR_ID: u16 = 0x4d9;
const PRODUCT_ID: u16 = 0xa052;

const SALT: [u8; 8] = [0u8; 8];

fn decode(salt: &[u8], data: &mut [u8]) {
    data.swap(0, 2);
    data.swap(1, 4);
    data.swap(3, 7);
    data.swap(5, 6);

    for i in 0..8 {
        data[i] ^= salt[i];
    }

    let tmp = data[7] << 5;
    data[7] = (data[6] << 5) | (data[7] >> 3);
    data[6] = (data[5] << 5) | (data[6] >> 3);
    data[5] = (data[4] << 5) | (data[5] >> 3);
    data[4] = (data[3] << 5) | (data[4] >> 3);
    data[3] = (data[2] << 5) | (data[3] >> 3);
    data[2] = (data[1] << 5) | (data[2] >> 3);
    data[1] = (data[0] << 5) | (data[1] >> 3);
    data[0] = tmp | (data[0] >> 3);

    let magic_word = "Htemp99e".as_bytes();

    for i in 0..8 {
        let b1 = data[i];
        let b2 = (magic_word[i] << 4) | (magic_word[i] >> 4);
        data[i] = ((b1 as i32 - b2 as i32) & 0xff) as u8;
    }
}

pub enum Value {
    Temperature(f64),
    Co2(u16),
    Unknown([u8; 8]),
}

pub struct Co2Mini {
    device: Device,
}

impl Co2Mini {
    pub fn open() -> Result<Co2Mini> {
        let mut device_infos = DeviceInfos::new(Some(VENDOR_ID), Some(PRODUCT_ID));
        let device_info = match device_infos.next() {
            Some(device_info) => device_info,
            None => {
                return Err(format_err!(
                    "Device with vendor if {:x} and product id {:x} not found",
                    VENDOR_ID,
                    PRODUCT_ID
                ))
            }
        };
        let device = Device::open_path(&device_info.path)?;

        device.send_feature_report(&SALT)?;

        Ok(Co2Mini { device })
    }

    pub fn read(&self) -> Result<Value> {
        let mut data = [0u8; 8];
        self.device.read(&mut data)?;
        decode(&SALT, &mut data);

        let value = match data[0] {
            0x42 => {
                let v = ((data[1] as u16) << 8) + data[2] as u16;
                let v = v as f64 * 0.0625 - 273.15;
                Value::Temperature(v)
            }
            0x50 => {
                let v = ((data[1] as u16) << 8) + data[2] as u16;
                Value::Co2(v)
            }
            _ => Value::Unknown(data),
        };

        Ok(value)
    }
}
