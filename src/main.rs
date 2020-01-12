#[macro_use]
extern crate common_failures;
#[macro_use]
extern crate failure;
extern crate hidapi_sys;
extern crate redis;
extern crate widestring;

mod hidapi;

use common_failures::prelude::*;

use hidapi::Device;
use hidapi::DeviceInfos;

quick_main!(run);

fn decode(magic_table: &[u8], data: &mut [u8]) {
    data.swap(0, 2);
    data.swap(1, 4);
    data.swap(3, 7);
    data.swap(5, 6);

    for i in 0..8 {
        data[i] ^= magic_table[i];
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

fn run() -> Result<()> {
    let vendor_id = 0x4d9;
    let product_id = 0xa052;

    let mut device_infos = DeviceInfos::new(Some(vendor_id), Some(product_id));
    let device_info = match device_infos.next() {
        Some(device_info) => device_info,
        None => {
            return Err(format_err!(
                "Device with vendor if {:x} and product id {:x} not found",
                vendor_id,
                product_id
            ))
        }
    };

    println!("{:?}", device_info);

    let device = Device::open_path(&device_info.path)?;

    let magic_table = [0u8; 8];
    device.send_feature_report(&magic_table)?;

    loop {
        let mut data = [0u8; 8];
        device.read(&mut data)?;

        decode(&magic_table, &mut data);

        let w = ((data[1] as u16) << 8) + data[2] as u16;

        if data[0] == 0x42 {
            let t = w as f64 * 0.0625 - 273.15;
            println!("Temp {}", t);
        }

        if data[0] == 0x50 {
            println!("CO2 {}", w);
        }
    }
}
