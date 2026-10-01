use std::fs::File;
use std::io::BufReader;

use rodio::{Decoder, DeviceSinkBuilder, MixerDeviceSink, Player};

/// Música de fondo en loop. Hay que mantenerla viva: al soltarla (`drop`)
/// se cierra el dispositivo de audio y la música se detiene.
pub struct BackgroundMusic {
    _sink: MixerDeviceSink,
    _player: Player,
}

impl BackgroundMusic {
    /// Abre la salida de audio por defecto y reproduce `path` en loop. Si no
    /// hay dispositivo de audio o falta el archivo, avisa por consola y
    /// devuelve `None`: el diorama sigue funcionando sin sonido.
    pub fn start(path: &str, volume: f32) -> Option<Self> {
        match Self::try_start(path, volume) {
            Ok(music) => Some(music),
            Err(err) => {
                eprintln!("[audio] no se pudo reproducir '{path}': {err}");
                None
            }
        }
    }

    fn try_start(path: &str, volume: f32) -> Result<Self, Box<dyn std::error::Error>> {
        let mut sink = DeviceSinkBuilder::open_default_sink()?;
        sink.log_on_drop(false);
        let player = Player::connect_new(sink.mixer());
        let file = BufReader::new(File::open(path)?);
        player.append(Decoder::new_looped(file)?);
        player.set_volume(volume);
        Ok(Self {
            _sink: sink,
            _player: player,
        })
    }
}
