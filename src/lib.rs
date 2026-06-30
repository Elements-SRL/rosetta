use tracing::instrument;

use crate::models::Calibration;

pub mod models;
pub mod syncro;
pub mod util;

#[instrument]
pub fn read_eeprom() {
    // trigger lettura eeprom (trigger 6): tutte le eeprom vengono lette contemporaneamente;
    // il contenuto di ogni eeprom viene salvato in una ram interna alla fpga;
    tracing::info!("eeprom read");
}

#[instrument]
pub fn get_ram(board_number: usize) {
    // trigger per inviare il contenuto di una ram al pc (trigger 7, viene inviato il contenuto di una sola ram).
    // In particolare, viene usato sempre il campo ram_cs per selezionare quale ram inviare.
    // Se ram_cs contiene piu di un bit alto, il contenuto inviato al pc non ha senso.
    tracing::info!("ram has been read");
}

// #[instrument]
// pub fn update_ram_address(ram_cs: address: u16, data: u8) {
//     // trigger per aggiornare la ram (trigger 8, si può aggiornare solo una ram alla volta):
// 	// settare il campo dell'address (16 bit, i 5 MSB vengono ignorati dalla eeprom) e il campo del dato (8 bit);
// 	// settare inoltre il campo ram_cs (24 bit), per selezionare quale ram aggiornare;
// 	// NOTA: ram_cs deve avere solo un bit alto, altrimenti lo stesso dato viene scritto in più ram.
// 	// ESEMPIO: ram_cs = "000 .. 001" --> aggiornare la prima ram;
// 	// 	        ram_cs = "100 .. 000" --> aggiornare l ultima ram;
//     //          ram_cs = "000 .. 011" --> vengono aggiornate sia la ram 1 che la ram 2, ma lo stesso dato nello stesso indirizzo.

// }

#[cfg(test)]
mod lib_tests {
    use tracing::instrument;

    use crate::{get_ram, read_eeprom};

    #[test]
    fn read_eeprom_test() {
        read_eeprom();
        get_ram(0);
    }
}
