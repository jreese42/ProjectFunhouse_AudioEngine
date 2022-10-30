use std::env;
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use paho_mqtt as mqtt;
use std::collections::VecDeque;
use std::time::{Duration, Instant};
use rodio::Source;
use std::thread;
use std::sync::Arc;

//////////////////////////////////////////////////////////////////////////
#[derive(Clone)]
struct TrackInfo {
    track_name: String,
    track_file: String,
    fade_in_secs: u64,
    audio_cues: VecDeque<AudioCue>,
}

#[derive(Clone)]
struct AudioCue {
    millis: u64,
    mqtt_data: String,
}

/* For simplicity in the code, add a trait extension to mqtt_client which allows queuing the exact messages we'll use */
pub trait MqttExt {
    fn notify_track_change(&self, track_name: &str);
    fn notify_audio_cue(&self, track_name: &str, cue_data: &str);
    fn set_simulated(&self);
    fn set_real(&self);
}

impl MqttExt for mqtt::AsyncClient {
    fn notify_track_change(&self, track_name: &str) {
        //Construct payload
        let mut payload_str = String::from("{\"track_name\":\"");
        payload_str.push_str(track_name);
        payload_str.push_str("\"}");

        let msg = mqtt::Message::new("audioEngine/trackChange", payload_str, mqtt::QOS_2);
        let tok = self.publish(msg);
        if let Err(e) = tok.wait() {
            println!("Error sending message: {:?}", e);
        }
        
        println!("to MQTT: Playing Track: {}", track_name);
    }
    
    fn notify_audio_cue(&self, track_name: &str, cue_data: &str) {
        //Construct payload
        let mut payload_str = String::from("{\"track_name\":\"");
        payload_str.push_str(track_name);
        payload_str.push_str("\",\"cue_data\":\"");
        payload_str.push_str(cue_data);
        payload_str.push_str("\"}");

        let msg = mqtt::Message::new("audioEngine/audioCue", payload_str, mqtt::QOS_2);
        let tok = self.publish(msg);
        if let Err(e) = tok.wait() {
            println!("Error sending message: {:?}", e);
        }

        println!("to MQTT: Audio Cue on {} => \"{}\"", track_name, cue_data);
    }

    fn set_simulated(&self) {
        //Construct payload
        let payload_str = String::from("{\"environment\":\"simulated\"}");

        let msg = mqtt::Message::new("audioEngine/environment", payload_str, mqtt::QOS_2);
        let tok = self.publish(msg);
        if let Err(e) = tok.wait() {
            println!("Error sending message: {:?}", e);
        }

        println!("to MQTT: Use Simulated Environment");
    }

    fn set_real(&self) {
        //Construct payload
        let payload_str = String::from("{\"environment\":\"real\"}");

        let msg = mqtt::Message::new("audioEngine/environment", payload_str, mqtt::QOS_2);
        let tok = self.publish(msg);
        if let Err(e) = tok.wait() {
            println!("Error sending message: {:?}", e);
        }

        println!("to MQTT: Use Real Environment");
    }
}

fn main() {
    let mut app_config = get_app_config();
    println!("{}", app_config.entry(String::from("soundsDirectory")).or_default());

    let mqtt_host : String;
    match env::var("MQTT_HOST") {
        Ok(val) => mqtt_host = val,
        Err(_e) => mqtt_host = "".to_string(),
    }

    let mqtt_user : String;
    match env::var("MQTT_USER") {
        Ok(val) => mqtt_user = val,
        Err(_e) => mqtt_user = "".to_string(),
    }

    let mqtt_pass : String;
    match env::var("MQTT_PASS") {
        Ok(val) => mqtt_pass = val,
        Err(_e) => mqtt_pass = "".to_string(),
    }

    let mqtt_client = connect_to_mqtt_server(&mqtt_host, &mqtt_user, &mqtt_pass).expect("Failed to connect to MQTT Broker");
    
    // Message receiver
    let rx = mqtt_client.start_consuming();

    // Subscribe to a topic
    mqtt_client.subscribe(String::from("audioEngine/input"), 0);
 
    // Get a output stream handle to the default physical sound device
    let (_stream, stream_handle) = rodio::OutputStream::try_default().expect("Failed ot get access to default audio device.");
    let arc_stream_handle = Arc::new(stream_handle);

    // Start waiting for messages
    let clone_stream_handle = arc_stream_handle.clone();
    let _mqtt_reader = thread::spawn(move || {
        loop {
        match rx.recv().expect("Error receiving message") {
            Some(_message) => {
                do_doorbell_event(&clone_stream_handle);
            }
            None => {}
            }
        }
    });

    // mqtt_client.set_simulated();
    mqtt_client.set_real();

    let mut playlist : VecDeque<TrackInfo> = VecDeque::new();

    // playlist.push_back(TrackInfo {track_name: String::from("delay"), track_file: String::from("sounds/silence_30s.ogg"), fade_in_secs: 2,
    //     audio_cues: VecDeque::from(vec![])   
    // }); //delay to run outside

    // playlist.push_back(TrackInfo {track_name: String::from("preshow"), track_file: String::from("sounds/silence_30s.ogg"), fade_in_secs: 2,
    //     audio_cues: VecDeque::from(vec![AudioCue {millis: 22000, mqtt_data: String::from("blackout")}])   
    // }); //start recording 15s after tree turns on

    playlist.push_back(TrackInfo {track_name: String::from("Scary Children"), track_file: String::from("sounds/scary_children.ogg"), fade_in_secs: 8,
        audio_cues: VecDeque::from(vec![])   
    });
    playlist.push_back(TrackInfo {track_name: String::from("Howling Wind"), track_file: String::from("sounds/howling_wind.ogg"), fade_in_secs: 6,
        audio_cues: VecDeque::from(vec![])   
    });
    playlist.push_back(TrackInfo {track_name: String::from("Church Tower Tolling"), track_file: String::from("sounds/church_tower_tolling_new.ogg"), fade_in_secs: 1,
        audio_cues: VecDeque::from(vec![AudioCue {millis: 22000, mqtt_data: String::from("tolling stopped")}])   
    });
    playlist.push_back(TrackInfo {track_name: String::from("Church Tower Tolling"), track_file: String::from("sounds/church_tower_tolling_new.ogg"), fade_in_secs: 1,
        audio_cues: VecDeque::from(vec![AudioCue {millis: 22000, mqtt_data: String::from("tolling stopped")}])   
    }); //double down
    playlist.push_back(TrackInfo {track_name: String::from("Two Weeks and Counting"), track_file: String::from("sounds/two_weeks_and_counting.ogg"), fade_in_secs: 2,
        audio_cues: VecDeque::from(vec![])   
    });
    playlist.push_back(TrackInfo {track_name: String::from("Spooky Story"), track_file: String::from("sounds/spooky_story.ogg"), fade_in_secs: 2,
        audio_cues: VecDeque::from(vec![])   
    });
    playlist.push_back(TrackInfo {track_name: String::from("Sinister Organ"), track_file: String::from("sounds/sinister_organ_short.ogg"), fade_in_secs: 2,
        audio_cues: VecDeque::from(vec![AudioCue {millis: 60500, mqtt_data: String::from("thunder")},
                                        AudioCue {millis: 99000, mqtt_data: String::from("heavy thunder")},
                                        AudioCue {millis: 119500, mqtt_data: String::from("storm")}])   
    });
    playlist.push_back(TrackInfo {track_name: String::from("Ghost Twins Singing"), track_file: String::from("sounds/ghost_twins_singing.ogg"), fade_in_secs: 2,
        audio_cues: VecDeque::from(vec![AudioCue {millis: 0, mqtt_data: String::from("up")},
                                        AudioCue {millis: 14500, mqtt_data: String::from("down")},
                                        AudioCue {millis: 18000, mqtt_data: String::from("up")},
                                        AudioCue {millis: 31500, mqtt_data: String::from("down")},
                                        AudioCue {millis: 34500, mqtt_data: String::from("up")},
                                        AudioCue {millis: 48000, mqtt_data: String::from("down")}])   
    });
    playlist.push_back(TrackInfo {track_name: String::from("Heartbeat"), track_file: String::from("sounds/heartbeat.ogg"), fade_in_secs: 2,
        audio_cues: VecDeque::from(vec![AudioCue {millis: 10000, mqtt_data: String::from("1")},
                                        AudioCue {millis: 19000, mqtt_data: String::from("2")},
                                        AudioCue {millis: 26000, mqtt_data: String::from("3")},
                                        AudioCue {millis: 30000, mqtt_data: String::from("4")}])   
    });
    playlist.push_back(TrackInfo {track_name: String::from("Marleys Footsteps"), track_file: String::from("sounds/marleys_footsteps.ogg"), fade_in_secs: 2,
        audio_cues: VecDeque::from(vec![AudioCue {millis: 49500, mqtt_data: String::from("door")},
                                        AudioCue {millis: 54500, mqtt_data: String::from("silent")},
                                        AudioCue {millis: 55750, mqtt_data: String::from("scream")}])     
    });
    playlist.push_back(TrackInfo {track_name: String::from("Michael Attacks"), track_file: String::from("sounds/michael_attacks.ogg"), fade_in_secs: 1,
        audio_cues: VecDeque::from(vec![AudioCue {millis: 700, mqtt_data: String::from("start")}])   
    });
    playlist.push_back(TrackInfo {track_name: String::from("Toccata and Fugue"), track_file: String::from("sounds/toccata_and_fugue.ogg"), fade_in_secs: 0,
        audio_cues: VecDeque::from(vec![])   
    });

    // playlist.push_back(TrackInfo {track_name: String::from("postshow"), track_file: String::from("sounds/silence_30s.ogg"), fade_in_secs: 2,
    //     audio_cues: VecDeque::from(vec![])   
    // }); //return to normal after show

    let sink = rodio::Sink::try_new(&arc_stream_handle.clone()).expect("Failed to create Audio Sink on default audio device.");
    sink.set_volume(0.33); //Turn down the music sink so the sfx play louder by comparison

    // let mut millis_accumulator = 0;
    let mut track_start_time = Instant::now();
    let mut audio_cues : VecDeque<AudioCue> = VecDeque::new();
    let mut next_audio_cue : Option<AudioCue> = None;
    let mut current_track_name = String::from("");
    loop {
        
        //No audio playing, so cue up a new track
        if sink.len() == 0 {
            if playlist.len() == 0 {
                std::process::exit(0); //No more tracks, exit program
            }

            let track_info = playlist.pop_front().unwrap();

            //requeue at the end of the playlist
            playlist.push_back(track_info.clone());
            
            // millis_accumulator = 0;
            
            let file = BufReader::new(File::open(track_info.track_file).unwrap());
            let music_source = rodio::Decoder::new(file).unwrap();
            let music_source = music_source.fade_in(Duration::from_secs(track_info.fade_in_secs));
            sink.append(music_source);
            current_track_name = String::from(&track_info.track_name);
            track_start_time = Instant::now();
            audio_cues = track_info.audio_cues;
            next_audio_cue = audio_cues.pop_front();
            mqtt_client.notify_track_change(track_info.track_name.as_str());
        }
                
        std::thread::sleep(Duration::from_millis(1));
        match &next_audio_cue {
            Some(cue) => {
                if track_start_time.elapsed().as_millis() >= cue.millis.into() {
                    //emit cue
                    mqtt_client.notify_audio_cue(&current_track_name, &cue.mqtt_data);
                    //grab next cue
                    next_audio_cue = audio_cues.pop_front();
                }
            },
            None => {}
        }

        // sink.set_volume(sink.volume() - 0.001);
    }

}

fn connect_to_mqtt_server(broker_address : &str, mqtt_user : &str, mqtt_pass : &str) -> Result<mqtt::AsyncClient, paho_mqtt::Error> {
    // Create the MQTT client
    println!("Connecting to MQTT Broker at {}", broker_address);

    let mqtt_client = mqtt::AsyncClient::new(broker_address)?;
    let conn_opts = mqtt::ConnectOptionsBuilder::new()
        .user_name(mqtt_user)
        .password(mqtt_pass)
        .finalize();
    // Connect and wait for it to complete or fail
    if let Err(e) = mqtt_client.connect(conn_opts).wait() {
        println!("Unable to connect: {:?}", e);
        std::process::exit(1);
    }

    Ok(mqtt_client)
}

fn do_doorbell_event(stream_handle : &Arc<rodio::OutputStreamHandle>) {
    println!("Doorbell Event");

    // let (_stream, stream_handle) = rodio::OutputStream::try_default().expect("Failed ot get access to default audio device.");
    // let sink = rodio::Sink::try_new(&stream_handle).expect("Failed to create Audio Sink on default audio device.");

    let file = BufReader::new(File::open(String::from("sounds/witch_cackle.mp3")).unwrap());
    let sndfx = stream_handle.play_once(BufReader::new(file)).unwrap();
    sndfx.detach();
    // let music_source = rodio::Decoder::new(file).unwrap();
    // sink.append(music_source);
}

fn get_app_config() -> HashMap<String, String> {
    let mut app_config = HashMap::new();
    let path = env::current_dir().unwrap_or(std::path::Path::new(".").to_path_buf());
    let sounds_directory_uri = std::path::Path::new("sounds");
    let sounds_path_as_string = String::from(path.join(sounds_directory_uri).to_str().unwrap_or("."));
    app_config.insert(String::from("soundsDirectory"), String::from(sounds_path_as_string));
    app_config
}
/* TODO */
//Open config file
    //Read sound directory from config
    //Read MQTT server info from config
    //synchronization offset millis
//Play simple audio
    //Read audio config file
        //Sound file
        //cues list
    //-play sound file-
//connect to mqtt server
    //-emit cues-
    //receive events
//events
    //play sound (option: crossfade)
    //play effect (option: concurrency = both, duck, pause w crossfade)
//commandline
    //play playlist
    //set config file
    //play single song
    //repeat
//playlist
    //read playlist
    //queue songs
    //looping

//nice to have
    //song config start at, end at