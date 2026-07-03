use std::ffi::{CStr, CString};

use tracing::instrument;
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

const E384_SUCCESS: E384Err = 0;
const E384_WARNING_VALUE_CLIPPED: E384Err = 0x8000_000B;

fn check(err: E384Err) -> Result<(), E384Err> {
    if err == E384_SUCCESS || err == E384_WARNING_VALUE_CLIPPED {
        Ok(())
    } else {
        Err(err)
    }
}


pub fn connect_to_first_device() -> Result<*mut E384Device, E384Err> {
     // ---- 1. Scan for devices --------------------------------------
    let mut list: *mut E384DeviceList = std::ptr::null_mut();
    check(unsafe { e384_detectDevices(&mut list) })?;

    let count = unsafe { e384_deviceList_count(list) };
    println!("Found {count} device(s):");

    let mut device_ids: Vec<String> = Vec::with_capacity(count);
    for i in 0..count {
        let ptr = unsafe { e384_deviceList_get(list, i) };
        if ptr.is_null() {
            continue;
        }
        // Copy the string out now — it's only valid until deviceList_free.
        let id = unsafe { CStr::from_ptr(ptr) }.to_string_lossy().into_owned();
        println!("  [{i}] {id}");
        device_ids.push(id);
    }

    unsafe { e384_deviceList_free(list) };

    let Some(first_id) = device_ids.first() else {
        println!("No devices to connect to.");
        return Err(E384_SUCCESS);
    };

    // ---- 2. Connect to the first one --------------------------------
    let c_id = CString::new(first_id.as_str()).expect("device id had an embedded NUL");
    let mut device: *mut E384Device = std::ptr::null_mut();
    check(unsafe { e384_connect(c_id.as_ptr(), &mut device) })?;
    println!("\nConnected to {first_id}");
    Ok(device)
}

#[instrument]
pub fn read_eeprom() {
    // okMoveCalibrationEepromToRams();
    // trigger lettura eeprom (trigger 6): tutte le eeprom vengono lette contemporaneamente;
    // il contenuto di ogni eeprom viene salvato in una ram interna alla fpga;
    tracing::info!("eeprom have been read");
}

#[instrument]
pub fn get_ram(board_number: u32) {
    // trigger per inviare il contenuto di una ram al pc (trigger 7, viene inviato il contenuto di una sola ram).
    // In particolare, viene usato sempre il campo ram_cs per selezionare quale ram inviare.
    // Se ram_cs contiene piu di un bit alto, il contenuto inviato al pc non ha senso.
    // okReadCalibrationRam(u16);
    tracing::info!("ram has been read");
}

#[instrument]
pub fn set_ram(board_number: u32) {
    //     virtual ErrorCodes_t okSelectCalibrationRam(uint16_t ramIdx);
    tracing::info!("ram has been set to work with");
}

#[instrument]
pub fn write_u8(device: Option<&mut E384Device>, address: u16, value: u8) {
    // unsafe {e384_okMoveCalibrationEepromToRams(device)};
    // okWriteCalibrationRam(uint16_t address, uint8_t value);
    tracing::trace!("writing...");
}


// /*! Copy the calibration EEPROM contents into the RAMs. */
// E384C_API E384Err e384_okMoveCalibrationEepromToRams(E384Device* device);

// /*! Copy the calibration RAMs back into the EEPROM. */
// E384C_API E384Err e384_okMoveCalibrationRamsToEeprom(E384Device* device);

// /*! Select the active calibration RAM by index. */
// E384C_API E384Err e384_okSelectCalibrationRam(E384Device* device, uint16_t ramIdx);

// /*! Write one byte to the selected calibration RAM at the given address. */
// E384C_API E384Err e384_okWriteCalibrationRam(E384Device* device,
//                                              uint16_t address,
//                                              uint8_t value);

#[instrument]
pub fn write_all_eeproms() {
    //  trigger per aggiornare la ram (trigger 8, si può aggiornare solo una ram alla volta):
    // 	settare il campo dell'address (16 bit, i 5 MSB vengono ignorati dalla eeprom) e il campo del dato (8 bit);
    // 	settare inoltre il campo ram_cs (24 bit), per selezionare quale ram aggiornare;
    // 	NOTA: ram_cs deve avere solo un bit alto, altrimenti lo stesso dato viene scritto in più ram.
    // 	ESEMPIO: ram_cs = "000 .. 001" --> aggiornare la prima ram;
    // 		     ram_cs = "100 .. 000" --> aggiornare l ultima ram;
    //           ram_cs = "000 .. 011" --> vengono aggiornate sia la ram 1 che la ram 2, ma lo stesso dato nello stesso indirizzo.
    //     virtual ErrorCodes_t okMoveCalibrationRamsToEeprom();
    tracing::trace!("writing all eeproms to apply calibrations");
}

#[cfg(test)]
mod e384_commands_tests {
    use crate::e384_commands::{get_ram, read_eeprom};

    #[test]
    fn read_eeprom_test() {
        read_eeprom();
        get_ram(0);
    }
}
