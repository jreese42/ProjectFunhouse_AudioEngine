use std::env;
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use paho_mqtt as mqtt;
use std::collections::VecDeque;
use std::time::{Duration, Instant};
use rodio::Source;

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

/* For simplicity in the code, add a trait extension to mqtt_client which allows queuing the exact message's we'll use */
pub trait MqttExt {
    fn notify_track_change(&self, track_name: &str);
    fn notify_audio_cue(&self, track_name: &str, cue_data: &str);
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
}

fn main() {
    let mut app_config = get_app_config();
    println!("{}", app_config.entry(String::from("soundsDirectory")).or_default());

    let mqtt_host = String::from("192.168.1.219:1883");
    let mqtt_client = connect_to_mqtt_server(&mqtt_host).expect("Failed to connect to MQTT Broker");

    let mut playlist : VecDeque<TrackInfo> = VecDeque::new();
    // playlist.push_back(TrackInfo {track_name: String::from("Spooky Story"), track_file: String::from("sounds/spooky_story.ogg")});
    // playlist.push_back(TrackInfo {track_name: String::from("Next Up Forever"), track_file: String::from("sounds/01 - Next Up Forever.flac")});
    // playlist.push_back(TrackInfo {track_name: String::from("Birthday Party"), track_file: String::from("sounds/02 - Birthday Party.flac")});
    // playlist.push_back(TrackInfo {track_name: String::from("Test One"), track_file: String::from("sounds/one.ogg")});
    // playlist.push_back(TrackInfo {track_name: String::from("Test Two"), track_file: String::from("sounds/two.ogg")});
    // playlist.push_back(TrackInfo {track_name: String::from("Test Three"), track_file: String::from("sounds/three.ogg")});
    // playlist.push_back(TrackInfo {track_name: String::from("Test Four"), track_file: String::from("sounds/four.ogg")});

    // playlist.push_back(TrackInfo {track_name: String::from("Scary Children"), track_file: String::from("sounds/scary_children.ogg"), fade_in_secs: 2,
    //     audio_cues: VecDeque::from(vec![])   
    // }); //done
    // playlist.push_back(TrackInfo {track_name: String::from("Howling Wind"), track_file: String::from("sounds/howling_wind.ogg"), fade_in_secs: 6,
    //     audio_cues: VecDeque::from(vec![])   
    // }); //done
    playlist.push_back(TrackInfo {track_name: String::from("Church Tower Tolling"), track_file: String::from("sounds/church_tower_tolling_new.ogg"), fade_in_secs: 1,
        audio_cues: VecDeque::from(vec![AudioCue {millis: 22000, mqtt_data: String::from("tolling stopped")}])   
    }); //done
    playlist.push_back(TrackInfo {track_name: String::from("Church Tower Tolling"), track_file: String::from("sounds/church_tower_tolling_new.ogg"), fade_in_secs: 1,
        audio_cues: VecDeque::from(vec![AudioCue {millis: 22000, mqtt_data: String::from("tolling stopped")}])   
    }); //double down
    playlist.push_back(TrackInfo {track_name: String::from("Two Weeks and Counting"), track_file: String::from("sounds/two_weeks_and_counting.ogg"), fade_in_secs: 2,
        audio_cues: VecDeque::from(vec![])   
    }); //done
    playlist.push_back(TrackInfo {track_name: String::from("Spooky Story"), track_file: String::from("sounds/spooky_story.ogg"), fade_in_secs: 2,
        audio_cues: VecDeque::from(vec![])   
    }); //done
    playlist.push_back(TrackInfo {track_name: String::from("Sinister Organ"), track_file: String::from("sounds/sinister_organ_short.ogg"), fade_in_secs: 2,
        audio_cues: VecDeque::from(vec![AudioCue {millis: 60500, mqtt_data: String::from("thunder")},
                                        AudioCue {millis: 99000, mqtt_data: String::from("heavy thunder")},
                                        AudioCue {millis: 119500, mqtt_data: String::from("storm")}])   
    }); //done
    playlist.push_back(TrackInfo {track_name: String::from("Ghost Twins Singing"), track_file: String::from("sounds/ghost_twins_singing.ogg"), fade_in_secs: 2,
        audio_cues: VecDeque::from(vec![AudioCue {millis: 0, mqtt_data: String::from("up")},
                                        AudioCue {millis: 14500, mqtt_data: String::from("down")},
                                        AudioCue {millis: 18000, mqtt_data: String::from("up")},
                                        AudioCue {millis: 31500, mqtt_data: String::from("down")},
                                        AudioCue {millis: 34500, mqtt_data: String::from("up")},
                                        AudioCue {millis: 48000, mqtt_data: String::from("down")}])   
    }); //done
    playlist.push_back(TrackInfo {track_name: String::from("Heartbeat"), track_file: String::from("sounds/heartbeat.ogg"), fade_in_secs: 2,
        audio_cues: VecDeque::from(vec![AudioCue {millis: 10000, mqtt_data: String::from("1")},
                                        AudioCue {millis: 19000, mqtt_data: String::from("2")},
                                        AudioCue {millis: 26000, mqtt_data: String::from("3")},
                                        AudioCue {millis: 30000, mqtt_data: String::from("4")}])   
    }); //done
    // playlist.push_back(TrackInfo {track_name: String::from("Ghostly Voices"), track_file: String::from("sounds/ghostly_voices.ogg"), fade_in_secs: 2,
    //     audio_cues: VecDeque::from(vec![])   
    // }); //skip?
    playlist.push_back(TrackInfo {track_name: String::from("Marleys Footsteps"), track_file: String::from("sounds/marleys_footsteps.ogg"), fade_in_secs: 2,
        audio_cues: VecDeque::from(vec![AudioCue {millis: 49500, mqtt_data: String::from("door")},
                                        AudioCue {millis: 54500, mqtt_data: String::from("silent")},
                                        AudioCue {millis: 55750, mqtt_data: String::from("scream")}])     
    }); //done
    playlist.push_back(TrackInfo {track_name: String::from("Michael Attacks"), track_file: String::from("sounds/michael_attacks.ogg"), fade_in_secs: 1,
        audio_cues: VecDeque::from(vec![AudioCue {millis: 700, mqtt_data: String::from("start")}])   
    }); //done
    // playlist.push_back(TrackInfo {track_name: String::from("Demonic Worship"), track_file: String::from("sounds/demonic_worship.ogg"), fade_in_secs: 2,
    //     audio_cues: VecDeque::from(vec![])   
    // }); //skip?
    // playlist.push_back(TrackInfo {track_name: String::from("The Alligator"), track_file: String::from("sounds/the_alligator.ogg"), fade_in_secs: 2,
    //     audio_cues: VecDeque::from(vec![])   
    // }); //skip?
    playlist.push_back(TrackInfo {track_name: String::from("Toccata and Fugue"), track_file: String::from("sounds/toccata_and_fugue.ogg"), fade_in_secs: 0,
        audio_cues: VecDeque::from(vec![])   
    }); //done

    // Get a output stream handle to the default physical sound device
    let (_stream, stream_handle) = rodio::OutputStream::try_default().expect("Failed ot get access to default audio device.");
    let sink = rodio::Sink::try_new(&stream_handle).expect("Failed to create Audio Sink on default audio device.");

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

fn connect_to_mqtt_server(broker_address : &str) -> Result<mqtt::AsyncClient, paho_mqtt::Error> {
    // Create the MQTT client
    println!("Connecting to MQTT Broker at {}", broker_address);
    let mqtt_client = mqtt::AsyncClient::new(broker_address)?;
    let conn_opts = mqtt::ConnectOptionsBuilder::new()
        .user_name(String::from("granbywled"))
        .password(String::from("vthokies"))
        .finalize();
    // Connect and wait for it to complete or fail
    if let Err(e) = mqtt_client.connect(conn_opts).wait() {
        println!("Unable to connect: {:?}", e);
        std::process::exit(1);
    }

    Ok(mqtt_client)
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