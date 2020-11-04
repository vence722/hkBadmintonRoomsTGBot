use phf::phf_map;
use futures::StreamExt;
use telegram_bot::*;
use chrono::{DateTime, Duration, Utc, Datelike};
use std::collections::HashMap;
use std::vec::Vec;
use serde::{Deserialize, Serialize};

static ROOM_NAMES: phf::Map<&'static str, &'static str> = phf_map! {
    "208" => "Shuen Wan Stadium"
};

static TIME_SLOTS: [&'static str;16] = [
    "7:00AM - 8:00AM",
    "8:00AM - 9:00AM",
    "9:00AM - 10:00AM",
    "10:00AM - 11:00AM",
    "11:00AM - 12:00PM",
    "12:00PM - 12:00PM",
    "1:00PM - 2:00PM",
    "2:00PM - 3:00PM",
    "3:00PM - 4:00PM",
    "4:00PM - 5:00PM",
    "5:00PM - 6:00PM",
    "6:00PM - 7:00PM",
    "7:00PM - 8:00PM",
    "8:00PM - 9:00PM",
    "9:00PM - 10:00PM",
    "10:00PM - 11:00PM",
];

static WEEKDAYS: [&'static str;7] = [
    "星期日",
    "星期一",
    "星期二",
    "星期三",
    "星期四",
    "星期五",
    "星期六",
];

#[derive(Debug, Serialize, Deserialize)]
struct RoomsInfoResult {
    data: Vec<RoomsInfo>
}

#[derive(Debug, Serialize, Deserialize)]
struct RoomsInfo {
    #[serde(rename = "freeCourts")]
    free_courts: Vec<Option<i32>>,
    venue: String,
    #[serde(rename = "numCourts")]
    num_courts: i32,
}

#[derive(Debug)]
struct AvailableRoomsInfo {
    date: DateTime<Utc>,
    venue: String,
    time_slot_id: usize,
}

impl AvailableRoomsInfo {
    fn format_string(&self) -> String {
        format!("{} {} {}", ROOM_NAMES[self.venue.as_str()], format!("{}({})", self.date.format("%Y/%m/%d"), WEEKDAYS[self.date.weekday().num_days_from_sunday() as usize]), TIME_SLOTS[self.time_slot_id])
    }

    fn format_rooms(rooms: &Vec<AvailableRoomsInfo>) -> String {
        let mut res = String::new();
        for room in rooms {
            res.push_str(room.format_string().as_str());
            res.push_str("\n");
        }
        return res;
    }
}

#[tokio::main]
async fn main() -> Result<(), telegram_bot::Error> {
    let bot = Api::new("1432788269:AAFJFvkjXQ3YbTcVyPI7AWCWl3ei49A4qeI");
    let mut stream = bot.stream();

    let mut chat_ids = HashMap::<ChatId, Option<i8>>::new();

    while let Some(update) = stream.next().await {
        let update = update?;
        if let UpdateKind::Message(ref message) = update.kind {
            if let MessageKind::Text { ref data, ..} =  message.kind {
                println!("<{}> - {}", message.from.username.as_ref().unwrap_or(&"unknown".to_string()), data);
            }
            let available_rooms = crawl_data().await.unwrap();
            bot.send(SendMessage::new(message.chat.id(), AvailableRoomsInfo::format_rooms(&available_rooms))).await?;
            chat_ids.insert(message.chat.id(), None);
        }
    }

    Ok(())
}

async fn crawl_data()->Result<Vec<AvailableRoomsInfo>, reqwest::Error> {
    let mut available_rooms = Vec::<AvailableRoomsInfo>::new();
    for i in 1..10 {
        let date = Utc::now() + Duration::days(i);
        let date_str = (&date).format("%Y%m%d").to_string();
        let url = String::from(format!("http://leisurely-badminton.ddns.net/leisure.ly/results/badminton/v8/date/Rooms.{}.json", date_str.clone()));
        let resp = reqwest::get(url.as_str()).await?;
        let body = resp.text().await?;
        let result: RoomsInfoResult = serde_json::from_str(body.as_str()).unwrap();

        for (_, room_info) in result.data.iter().filter(|info| info.venue == "208").enumerate() {
            for (i, _) in room_info.free_courts.iter().filter(|count| count.unwrap_or(0) > 0).enumerate() {
                let available_room = AvailableRoomsInfo {
                    date,
                    venue: room_info.venue.clone(),
                    time_slot_id: i,
                };
                available_rooms.push(available_room);
            }
        }
    }
    Ok(available_rooms)
}
