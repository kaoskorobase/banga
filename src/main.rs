mod methcla;
use jack;
use sodium_rust::sodium::gc::NoGc;
use sodium_rust::sodium::SodiumCtx;
use sodium_rust::sodium::StreamSink;
use std::sync::mpsc;
use wmidi::MidiMessage;

fn main() {
    // open client
    let (client, _status) =
        jack::Client::new("banga", jack::ClientOptions::NO_START_SERVER).unwrap();

    // process logic
    let midi_in_port = client
        .register_port("midi_in", jack::MidiIn::default())
        .unwrap();

    let (tx, rx) = mpsc::channel();

    let cback = move |_: &jack::Client, ps: &jack::ProcessScope| -> jack::Control {
        let show_p = midi_in_port.iter(ps);
        for e in show_p {
            match MidiMessage::from_bytes(e.bytes) {
                Ok(msg) => match msg.drop_sysex() {
                    Some(msg) => {
                        // println!("{:?}", msg);
                        tx.send(msg).unwrap();
                    }
                    None => (),
                },
                Err(err) => println!("{:?}", err),
            }
        }
        jack::Control::Continue
    };

    // activate
    let active_client = client
        .activate_async((), jack::ClosureProcessHandler::new(cback))
        .unwrap();

    let mut sodium_ctx = SodiumCtx::new();
    let sodium_ctx = &mut sodium_ctx;
    let midi_s: StreamSink<NoGc<MidiMessage<'static>>> = sodium_ctx.new_stream_sink();
    let out_l = midi_s.to_stream().listen(move |a| println!("{:?}", **a));

    loop {
        let msg = rx.recv().unwrap();
        midi_s.send(&NoGc::new(msg));
    }

    out_l.unlisten();

    // optional deactivation
    active_client.deactivate().unwrap();
}
