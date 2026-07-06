use std::ffi::{CStr, CString};

use tracing::instrument;
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

const E384_SUCCESS: E384Err = 0;
const E384_WARNING_VALUE_CLIPPED: E384Err = 0x8000_000B;

#[instrument]
fn check(err: E384Err) -> Result<(), E384Err> {
    if err == E384_SUCCESS {
        tracing::trace!("Success!");
        Ok(())
    } else if err == E384_WARNING_VALUE_CLIPPED {
        tracing::warn!("Value clipped");
        Ok(())
    } else {
        tracing::error!("Error receviced with value: {}", err);
        Err(err)
    }
}

#[derive(Debug)]
pub struct E384MiniWrapper(*mut E384Device);

impl E384MiniWrapper {
    #[instrument]
    pub fn list_devices() -> Result<Vec<String>, E384Err> {
        let mut list: *mut E384DeviceList = std::ptr::null_mut();
        check(unsafe { e384_detectDevices(&mut list) })?;

        let count = unsafe { e384_deviceList_count(list) };
        tracing::info!("Found {count} device(s):");

        let mut device_ids: Vec<String> = Vec::with_capacity(count);
        for i in 0..count {
            let ptr = unsafe { e384_deviceList_get(list, i) };
            if ptr.is_null() {
                continue;
            }
            // Copy the string out now — it's only valid until deviceList_free.
            let id = unsafe { CStr::from_ptr(ptr) }
                .to_string_lossy()
                .into_owned();
            device_ids.push(id);
        }

        unsafe { e384_deviceList_free(list) };

        Ok(device_ids)
    }

    #[instrument]
    pub fn connect_to(id: &str) -> Result<Self, E384Err> {
        let c_id = CString::new(id).expect("device id had an embedded NUL");
        let mut device: *mut E384Device = std::ptr::null_mut();
        check(unsafe { e384_connect(c_id.as_ptr(), &mut device) })?;
        tracing::info!("Connected to {id}");
        Ok(Self(device))
    }

    #[instrument]
    pub fn read_eeprom(&mut self) {
        // /*! Copy the calibration EEPROM contents into the RAMs. */
        unsafe {
            let _ = check(e384_okMoveCalibrationEepromToRams(self.0));
        };
        // okMoveCalibrationEepromToRams();
        // trigger lettura eeprom (trigger 6): tutte le eeprom vengono lette contemporaneamente;
        // il contenuto di ogni eeprom viene salvato in una ram interna alla fpga;
        tracing::info!("eeprom have been read");
    }

    
    #[instrument]
    pub fn set_ram(&mut self, board_number: u16) {
        // /*! Select the active calibration RAM by index. */
        unsafe {
            let _ = check(e384_okSelectCalibrationRam(self.0, board_number));
        };
        tracing::info!("ram has been set to work with");
    }

    #[instrument(level = "trace")]
    pub fn write_u8(&mut self, address: u16, value: u8) {
        unsafe {
            let _ = check(e384_okWriteCalibrationRam(self.0, address, value));
        };
        tracing::trace!("writing...");
    }
    

    #[instrument(level = "trace")]
    pub fn write_all_eeproms(&mut self) {
        //  trigger per aggiornare la ram (trigger 8, si può aggiornare solo una ram alla volta):
        // 	settare il campo dell'address (16 bit, i 5 MSB vengono ignorati dalla eeprom) e il campo del dato (8 bit);
        // 	settare inoltre il campo ram_cs (24 bit), per selezionare quale ram aggiornare;
        // 	NOTA: ram_cs deve avere solo un bit alto, altrimenti lo stesso dato viene scritto in più ram.
        // 	ESEMPIO: ram_cs = "000 .. 001" --> aggiornare la prima ram;
        // 		     ram_cs = "100 .. 000" --> aggiornare l ultima ram;
        //           ram_cs = "000 .. 011" --> vengono aggiornate sia la ram 1 che la ram 2, ma lo stesso dato nello stesso indirizzo.
        //     virtual ErrorCodes_t okMoveCalibrationRamsToEeprom();

        // /*! Copy the calibration RAMs back into the EEPROM. */
        unsafe {
            let _ = check(e384_okMoveCalibrationRamsToEeprom(self.0));
        };
        tracing::trace!("writing all eeproms to apply calibrations");
    }
}

// #[instrument]
// pub fn get_ram(dev: *mut E384Device, board_number: u32) {
//     unsafe {
//         let e = e384_okMoveCalibrationEepromToRams(dev);
//         match e {
//             0 => tracing::info!("Succesfully moved calibration eeprom to rams"),
//             _ => tracing::error!("Error while moving calibration eeprom to rams: {}", e)
//         }
//     };
//     // trigger per inviare il contenuto di una ram al pc (trigger 7, viene inviato il contenuto di una sola ram).
//     // In particolare, viene usato sempre il campo ram_cs per selezionare quale ram inviare.
//     // Se ram_cs contiene piu di un bit alto, il contenuto inviato al pc non ha senso.
//     // okReadCalibrationRam(u16);
//     tracing::info!("ram has been read");
// }



// #[cfg(test)]
// mod e384_commands_tests {
//     use crate::e384_commands::read_eeprom;

//     // #[test]
//     // fn read_eeprom_test() {
//     //     read_eeprom();
//     // }
// }
