use tracing::instrument;

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

#[cfg(test)]
mod e384_commands_tests {
    use tracing::instrument;

    use crate::e384_commands::{get_ram, read_eeprom};

    #[test]
    fn read_eeprom_test() {
        read_eeprom();
        get_ram(0);
    }
}
