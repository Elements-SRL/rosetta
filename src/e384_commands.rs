use tracing::instrument;

#[instrument]
pub fn read_eeprom() {
    // trigger lettura eeprom (trigger 6): tutte le eeprom vengono lette contemporaneamente;
    // il contenuto di ogni eeprom viene salvato in una ram interna alla fpga;
    tracing::info!("eeprom read");
}

#[instrument]
pub fn get_ram(board_number: u32) {
    // trigger per inviare il contenuto di una ram al pc (trigger 7, viene inviato il contenuto di una sola ram).
    // In particolare, viene usato sempre il campo ram_cs per selezionare quale ram inviare.
    // Se ram_cs contiene piu di un bit alto, il contenuto inviato al pc non ha senso.
    tracing::info!("ram has been read");
}

#[instrument]
pub fn set_ram(board_number: u32) {
    tracing::info!("ram has been set to work with");
}

#[instrument]
pub fn write_u8(address: u16, value: u8) {
    tracing::trace!("writing...");
}

#[instrument]
pub fn write_all_eeproms() {
    //  trigger per aggiornare la ram (trigger 8, si può aggiornare solo una ram alla volta):
    // 	settare il campo dell'address (16 bit, i 5 MSB vengono ignorati dalla eeprom) e il campo del dato (8 bit);
    // 	settare inoltre il campo ram_cs (24 bit), per selezionare quale ram aggiornare;
    // 	NOTA: ram_cs deve avere solo un bit alto, altrimenti lo stesso dato viene scritto in più ram.
    // 	ESEMPIO: ram_cs = "000 .. 001" --> aggiornare la prima ram;
    // 		     ram_cs = "100 .. 000" --> aggiornare l ultima ram;
    //           ram_cs = "000 .. 011" --> vengono aggiornate sia la ram 1 che la ram 2, ma lo stesso dato nello stesso indirizzo.
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
