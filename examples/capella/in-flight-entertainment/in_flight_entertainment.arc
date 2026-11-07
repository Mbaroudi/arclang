// In-Flight Entertainment System: the reference sample shipped with Eclipse
// Capella (https://github.com/eclipse-capella/capella, samples/, EPL-2.0),
// imported as is with `arclang import -f capella` on 2026-10-18. Five levels
// (operational analysis to EPBS), 1 600 declarations: functions and their
// ports at every level, components and actors with their exchanges, physical
// nodes and links, functional chains, capabilities, state machines,
// scenarios with fragments, the data model and the requirements, plus the
// 118 diagrams of the .aird as `diagram` declarations. The ids are the
// Capella ids, so `arclang export -f capella` gives the model back.
model "In-Flight Entertainment System" {
    name: "In-Flight Entertainment System"
    description: "Imported from Capella (In-Flight Entertainment System.capella)"
}

data_type "String" {
    id: "0abcd19d-aad8-475a-a3a4-bb5f3553ea2e"
    base: "String"
}
data_type "Boolean" {
    id: "57e53d45-596c-48a4-b6fc-d6af808e5370"
    base: "Boolean"
}
enumeration "CLASS" {
    id: "6bca4516-9dc6-45ad-9302-d7924289aa23"
    values: ["BUSINESS", "PREMIUM", "ECONOMY"]
}
enumeration "Language" {
    id: "69950a66-33d4-4dfb-85b3-7b25046b4262"
    values: ["ENGLISH", "FRENCH", "CHINESE"]
}
data_type "Int" {
    id: "72e52ac3-59ae-4efb-aa3f-e08cae94f0be"
    base: "Integer"
}
data_type "ID" {
    id: "ddd9f5bf-6737-4b47-ba1b-a3ed13a45bea"
    base: "Integer"
}
data_type "Duration" {
    id: "ff210759-5ddf-4f3e-8556-5e16b54f7f94"
    base: "Real"
    unit: "s"
}
enumeration "STREAM_CONTROL" {
    id: "e5bebd43-6881-4c89-be4a-7cc97632df51"
    values: ["PAUSE", "STOP", "PLAY"]
}
data_type "DateTime" {
    id: "becb6a73-5c0d-44ad-9d0f-c2f610b48107"
    base: "Real"
    unit: "s"
}
data_type "MegaByte" {
    id: "ee902fdf-daee-46e7-ab0c-5cb3e61385b2"
    base: "Integer"
}
enumeration "VIDEO _FORMAT" {
    id: "10678425-5baa-47de-8d6b-e7ec0a47666f"
    values: ["720p", "1080i", "1080p"]
}
enumeration "CODEC" {
    id: "6105a469-c022-4fd3-86aa-892d8c0b5967"
    values: ["H264", "OGG", "MP3", "FLAC"]
}
enumeration "AUDIO_CHANNEL" {
    id: "5865788f-9133-42d8-8cfc-8b7b31388859"
    values: ["MONO", "STEREO", "5.1"]
}
data_type "Frame Rate" {
    id: "bad2a514-b52f-4c98-9be1-661a1f2177e2"
    base: "Real"
    unit_name: "fps"
}
data_type "Sample Rate" {
    id: "7fc204dc-9c56-4b00-a927-82a23b8bb5ca"
    base: "Real"
    unit_name: "sps"
}
class "Seat" {
    id: "0c17402c-e5e4-44d4-9692-efc4bfb8ab2a"
    class: "CLASS"
    field_2: "Screen"
}
class "Passenger" {
    id: "69bc8e28-37ec-4b2b-b47d-0cfb9ea8e645"
    seat: "Seat"
    name: "String"
    firstname: "String"
    field_4: "Seat"
    field_5: "Screen"
}
class "Cabin" {
    id: "8a3c3a57-1ff3-4d9e-845e-0e142c1ca7dc"
    crew: "Crew Member"
    passenger: "Passenger"
    field_3: "Speaker"
    field_4: "Screen"
}
class "Aircraft" {
    id: "67bd0af1-6242-4dec-aeef-3f54a7d8ba47"
    cabin: "Cabin"
    field_2: "Cabin"
    field_3: "Passenger"
    field_4: "Crew Member"
}
class "Crew Member" {
    id: "bcc88a58-3c68-4f46-8502-f5783a6f9046"
}
class "Speaker" {
    id: "4bb53659-1f4f-4fc0-9eb6-8e6d7bec1415"
}
class "Screen" {
    id: "e3b6bcb8-d453-489f-afb1-3c7811acad55"
}
class "Media" {
    id: "a990b5ef-faa7-453d-b1f0-d46e8a15c4fd"
    id_field: "ID"
    isimposed: "Boolean"
}
class "Audio File" {
    id: "74a5ce53-1394-4821-9cbf-c880d83a07dd"
    encoding: "Data"
}
class "Audio-Video File" {
    id: "dd056c2f-d8b0-4f9e-b4ba-382926d2aed4"
}
class "Frame" {
    id: "0ba002c1-c0d9-4e1e-98cf-60e55d02ecfd"
    number: "Int"
}
class "Movie" {
    id: "6c858859-acc3-4f31-a6fc-0aee4f8c07d8"
    extends: "Media"
    title: "String"
    description_field: "String"
    availablelanguage: "Language"
    frame: "Frame"
    file: "Audio-Video File"
}
class "Stream" {
    id: "44f6d074-c280-4d8d-83e0-ced6ba8d5cf9"
    movie: "Movie"
}
class "Control Command" {
    id: "2a07f10a-8c7b-4065-8d35-bf2b540c8348"
    extends: "Command"
    movie: "Movie"
    control: "STREAM_CONTROL"
    selection: "Interactor"
    resume: "Boolean"
}
class "Interactor" {
    id: "b417cf33-5d66-4249-9949-f547f7876a19"
    id_field: "ID"
    activated: "Boolean"
}
class "Command" {
    id: "6a78f28a-a27b-4f42-a002-04924ee4c75f"
    id_field: "ID"
}
class "VOD Control Command" {
    id: "a7059561-22f7-4589-aed8-05b33d8600c8"
    extends: "Control Command"
    currentposition: "Frame"
}
class "Seat (2)" {
    id: "fbed3075-3536-4603-9089-f9571423c7cd"
    id_field: "ID"
    class: "CLASS"
}
class "Imposed Video Control Command" {
    id: "dcb7f0b0-c6f6-4e96-b7bd-762bb0488430"
    extends: "Control Command"
}
class "Control Command (2)" {
    id: "e5d905f2-f0a3-48b6-9658-cecb923e6fb3"
    extends: "Command (2)"
    control: "STREAM_CONTROL"
    resume: "Boolean"
    selection: "UI Interactor"
    media: "Media (2)"
}
class "UI Interactor" {
    id: "d301f697-5975-41e2-97af-2aea81db5f6f"
    id_field: "ID"
    activated: "Boolean"
}
class "Command (2)" {
    id: "e02cf446-19e2-4b24-b72d-340fd3c39355"
    id_field: "ID"
}
class "VOD Control Command (2)" {
    id: "2e048fef-989b-4332-ae3c-e47c63bbec2b"
    extends: "Control Command (2)"
    currentposition: "Frame (2)"
}
class "Seat (3)" {
    id: "c9e7c37a-e940-434c-9dab-1f1029f2cf14"
    id_field: "ID"
    class: "CLASS"
}
class "Media (2)" {
    id: "93f522c4-99eb-40b2-9baa-a576b6a3af30"
    id_field: "ID"
    isimposed: "Boolean"
}
class "Audio File (2)" {
    id: "1d278944-5217-4004-a90d-c38a21cfd0a9"
    encoding: "CODEC"
}
class "Media File" {
    id: "eb58eec3-b4ee-4a51-80c2-666fae882c3f"
    videoencoding: "CODEC"
    videoformat: "VIDEO _FORMAT"
    audioencoding: "CODEC"
    isencoded: "Boolean"
    size: "MegaByte"
}
class "Frame (2)" {
    id: "8b064948-3736-4410-ab07-4d09ca2c1d24"
    number: "Int"
}
class "Movie (2)" {
    id: "940ce3fc-ef6f-4db0-85c2-d9866a9472a7"
    extends: "Media (2)"
    title: "String"
    description_field: "String"
    availablelanguage: "Language"
    frame: "Frame (2)"
    file: "Media File"
}
class "Video Stream Header" {
    id: "fbfc173b-0081-4501-9d92-f026ff210ef2"
    codec: "CODEC"
    language: "Language"
    videoformat: "VIDEO _FORMAT"
    rate: "Frame Rate"
}
class "Audio Packet" {
    id: "30f1f33a-1c0d-4c1f-8ada-62f22d676917"
    isencoded: "Boolean"
    audioencoding: "CODEC"
    timestamp: "DateTime"
}
class "Video Packet" {
    id: "36986419-6217-426f-bcab-e9ad33de0a80"
    isencoded: "Boolean"
    videoencoding: "CODEC"
    timestamp: "DateTime"
}
class "Audio Video Packet" {
    id: "89d4b45e-af13-4733-aa66-44790e984572"
    audio: "Audio Packet"
    video: "Video Packet"
}
class "Audio Stream Header" {
    id: "7fef079a-4ee1-44ea-9388-ecaefb276599"
    codec: "CODEC"
    channel: "AUDIO_CHANNEL"
    sample_rate: "Sample Rate"
}
association "has" {
    id: "0945715c-8c26-495b-ab84-8dd5f86ce7cb"
    from: "8a3c3a57-1ff3-4d9e-845e-0e142c1ca7dc"
    to: "4bb53659-1f4f-4fc0-9eb6-8e6d7bec1415"
    role: "field_3"
    kind: "composition"
    multiplicity: "1..*"
}
association "has" {
    id: "13f103e4-9abd-4b3d-bf89-3f1179a6123a"
    from: "8a3c3a57-1ff3-4d9e-845e-0e142c1ca7dc"
    to: "e3b6bcb8-d453-489f-afb1-3c7811acad55"
    role: "field_4"
    kind: "composition"
    multiplicity: "1..*"
}
association "contains" {
    id: "e5bb2184-5036-4be1-b292-686444f0a580"
    from: "0c17402c-e5e4-44d4-9692-efc4bfb8ab2a"
    to: "e3b6bcb8-d453-489f-afb1-3c7811acad55"
    role: "field_2"
    multiplicity: "1"
}
association "has" {
    id: "714bb1d1-f0ca-4efe-98c2-c859dbed1429"
    from: "67bd0af1-6242-4dec-aeef-3f54a7d8ba47"
    to: "8a3c3a57-1ff3-4d9e-845e-0e142c1ca7dc"
    role: "field_2"
    kind: "composition"
    multiplicity: "1"
}
association "transports" {
    id: "e7110273-d206-4ed1-8ea4-ea190fae610b"
    from: "67bd0af1-6242-4dec-aeef-3f54a7d8ba47"
    to: "69bc8e28-37ec-4b2b-b47d-0cfb9ea8e645"
    role: "field_3"
    kind: "composition"
    multiplicity: "1..*"
}
association "is operated by" {
    id: "f5cfa0a5-65ea-459c-b272-7cd0116975a7"
    from: "67bd0af1-6242-4dec-aeef-3f54a7d8ba47"
    to: "bcc88a58-3c68-4f46-8502-f5783a6f9046"
    role: "field_4"
    kind: "composition"
    multiplicity: "1..*"
}
association "is sitting on" {
    id: "f51b01d8-b6ad-4d8a-9820-da77e3542b1c"
    from: "69bc8e28-37ec-4b2b-b47d-0cfb9ea8e645"
    to: "0c17402c-e5e4-44d4-9692-efc4bfb8ab2a"
    role: "field_4"
    multiplicity: "1"
}
association "watches / plays with" {
    id: "b6564e56-0a50-4427-abb2-10f049f47d41"
    from: "69bc8e28-37ec-4b2b-b47d-0cfb9ea8e645"
    to: "e3b6bcb8-d453-489f-afb1-3c7811acad55"
    role: "field_5"
    multiplicity: "1"
}
association "DataAssociation6" {
    id: "8768d20a-5360-480b-939b-0d9ad037e869"
    from: "2a07f10a-8c7b-4065-8d35-bf2b540c8348"
    to: "6c858859-acc3-4f31-a6fc-0aee4f8c07d8"
    role: "movie"
    multiplicity: "1"
}
association "DataAssociation8" {
    id: "5c672ee8-f8e2-46ab-bf2c-e1f05ce3c965"
    from: "44f6d074-c280-4d8d-83e0-ced6ba8d5cf9"
    to: "6c858859-acc3-4f31-a6fc-0aee4f8c07d8"
    role: "movie"
    multiplicity: "1"
}
association "CommandAssociation2" {
    id: "c006549b-a3ae-4a35-9fc0-dca569507098"
    from: "a7059561-22f7-4589-aed8-05b33d8600c8"
    to: "0ba002c1-c0d9-4e1e-98cf-60e55d02ecfd"
    role: "currentposition"
    multiplicity: "1"
}
association "MediaAssociation1" {
    id: "3e727d6a-88ad-4ab0-88e9-93315aeaf9e1"
    from: "6c858859-acc3-4f31-a6fc-0aee4f8c07d8"
    to: "dd056c2f-d8b0-4f9e-b4ba-382926d2aed4"
    role: "file"
    kind: "composition"
    multiplicity: "1..*"
}
association "CommandAssociation1" {
    id: "156c5d02-6f81-4d8f-9abb-cf8a8412b717"
    from: "2a07f10a-8c7b-4065-8d35-bf2b540c8348"
    to: "b417cf33-5d66-4249-9949-f547f7876a19"
    role: "selection"
    multiplicity: "1"
}
association "CommandAssociation3" {
    id: "500cc79a-f76d-499e-83ae-8a869d381cc6"
    from: "6c858859-acc3-4f31-a6fc-0aee4f8c07d8"
    to: "0ba002c1-c0d9-4e1e-98cf-60e55d02ecfd"
    role: "frame"
    multiplicity: "1..*"
}
exchange_item "Audio-Video Stream" {
    id: "f1309e1f-e30b-47ed-8027-41f12017ef50"
    mechanism: "FLOW"
    elements: ["44f6d074-c280-4d8d-83e0-ced6ba8d5cf9"]
}
exchange_item "Media ID" {
    id: "abe9bb46-3a60-4f88-a4ad-42f99f1238de"
    mechanism: "FLOW"
    elements: ["6c858859-acc3-4f31-a6fc-0aee4f8c07d8"]
}
exchange_item "Audio-Video Broadcast Data" {
    id: "a86aeed3-084c-4618-b23a-bd385f92ecb0"
    mechanism: "FLOW"
    elements: ["fbed3075-3536-4603-9089-f9571423c7cd", "6c858859-acc3-4f31-a6fc-0aee4f8c07d8"]
}
exchange_item "Passenger Selection" {
    id: "e3830e92-47f9-4a57-b9ac-708bc64f6937"
    mechanism: "FLOW"
    elements: ["b417cf33-5d66-4249-9949-f547f7876a19"]
}
exchange_item "VOD Control Command (3)" {
    id: "1885aa4c-b56f-48cb-a433-a135f0bd7b01"
    mechanism: "FLOW"
    elements: ["a7059561-22f7-4589-aed8-05b33d8600c8", "fbed3075-3536-4603-9089-f9571423c7cd"]
}
exchange_item "Media Files" {
    id: "1fdba7f2-be01-4882-b999-b65528c3ee35"
    mechanism: "FLOW"
    elements: ["dd056c2f-d8b0-4f9e-b4ba-382926d2aed4"]
}
exchange_item "Imposed Video Control Command (2)" {
    id: "ba2b52d2-9f7a-4f53-9e44-24c00af74a53"
    mechanism: "FLOW"
    elements: ["dcb7f0b0-c6f6-4e96-b7bd-762bb0488430"]
}
exchange_item "Audio-Video Packets" {
    id: "b65b0dee-34a9-4397-9626-e8095e348482"
    mechanism: "FLOW"
    elements: ["89d4b45e-af13-4733-aa66-44790e984572"]
}
exchange_item "Audio Packets" {
    id: "cb931585-e5a5-48ba-8ac8-cf3b13fffe33"
    mechanism: "FLOW"
    elements: ["30f1f33a-1c0d-4c1f-8ada-62f22d676917"]
}
exchange_item "Video Packets" {
    id: "911f07f3-4c41-4351-aff4-1f1a84d9417a"
    mechanism: "FLOW"
    elements: ["36986419-6217-426f-bcab-e9ad33de0a80"]
}
exchange_item "Media File (2)" {
    id: "a34141d6-27d6-4e31-9994-828ee1560ded"
    mechanism: "FLOW"
    elements: ["eb58eec3-b4ee-4a51-80c2-666fae882c3f"]
}
exchange_item "Audio-Video Stream Header" {
    id: "b1ca4bf5-1b54-4f5c-a385-9514227695d3"
    mechanism: "FLOW"
    elements: ["fbfc173b-0081-4501-9d92-f026ff210ef2", "7fef079a-4ee1-44ea-9388-ecaefb276599"]
}
exchange_item "Media ID (2)" {
    id: "32703aff-a875-41e8-a93c-20e8a4b8df4b"
    mechanism: "FLOW"
    elements: ["940ce3fc-ef6f-4db0-85c2-d9866a9472a7"]
}
exchange_item "Media Broadcast Data" {
    id: "03c86c27-a965-4b3a-b641-e1969d4c7261"
    mechanism: "FLOW"
    elements: ["c9e7c37a-e940-434c-9dab-1f1029f2cf14", "940ce3fc-ef6f-4db0-85c2-d9866a9472a7"]
}
exchange_item "Passenger Selection (2)" {
    id: "1cd707ba-fe65-4e3c-9fff-21966d4e5716"
    mechanism: "FLOW"
    elements: ["d301f697-5975-41e2-97af-2aea81db5f6f"]
}
exchange_item "VOD Control Command (4)" {
    id: "d3b096f5-9eb4-4c4a-b0af-946a71992c54"
    mechanism: "FLOW"
    elements: ["2e048fef-989b-4332-ae3c-e47c63bbec2b", "c9e7c37a-e940-434c-9dab-1f1029f2cf14"]
}
exchange_item "Video Frames" {
    id: "a4ecc90d-72fb-486d-8de2-90d5473df459"
    mechanism: "FLOW"
    elements: ["8b064948-3736-4410-ab07-4d09ca2c1d24"]
}

operational_analysis "Operational Analysis" {
    id: "e12de7af-bf77-4c43-9f7c-03404be33a29"
    actor "Ground Operator" {
        id: "d7a95c22-1622-41d5-8201-37cd463075c7"
    }
    entity "Crew" {
        id: "e1b91d3f-beb2-40fd-869a-ad45ab8ec951"
    }
    actor "Cabin Crew" {
        id: "87bfb1b6-b85e-4497-950d-663dd78c4502"
        part_of: "e1b91d3f-beb2-40fd-869a-ad45ab8ec951"
    }
    actor "Pilot" {
        id: "c4fd192c-6826-4924-bbc3-3a4286b658b4"
        part_of: "e1b91d3f-beb2-40fd-869a-ad45ab8ec951"
    }
    entity "Aircraft" {
        id: "dab8526e-cf3a-4c8a-8fa1-7df4aa281bcf"
    }
    entity "Airline Company" {
        id: "be0d3f06-3e9b-4162-9ac6-2b16663d1dfa"
    }
    entity "Personal Device" {
        id: "dccf58d4-fe45-4a13-bb91-dc1b0c1cc0f5"
    }
    actor "Passenger" {
        id: "2060479f-e59f-4b8f-975f-f05f5986e9b3"
    }
    operational_activity "Provide Aircraft Localization" {
        id: "426f80c2-78ad-4e94-8e5b-da19f22aca26"
        performed_by: "dab8526e-cf3a-4c8a-8fa1-7df4aa281bcf"
    }
    operational_activity "Use Entertainment Services" {
        id: "79dd23d7-af39-4b79-892e-2f8fefbff5ff"
        operational_activity "Watch Movie" {
            id: "f9cabb24-100b-4e68-930a-8f35ec38c106"
            performed_by: "2060479f-e59f-4b8f-975f-f05f5986e9b3"
        }
        operational_activity "Listen to Audio" {
            id: "60dc5173-5420-44c0-bedf-ad3f6958a79d"
            performed_by: "2060479f-e59f-4b8f-975f-f05f5986e9b3"
        }
        operational_activity "Browse the Internet" {
            id: "d26f381c-470d-4e9f-a868-cca9bd4a76f6"
            performed_by: "2060479f-e59f-4b8f-975f-f05f5986e9b3"
        }
        operational_activity "Watch Moving Map" {
            id: "45857975-71b5-4d8c-be67-0edb138873d0"
            performed_by: "2060479f-e59f-4b8f-975f-f05f5986e9b3"
        }
        operational_activity "Play Games" {
            id: "4f4bb114-4fa9-4b49-aa38-11f7af1e86b1"
            performed_by: "2060479f-e59f-4b8f-975f-f05f5986e9b3"
        }
    }
    operational_activity "Define business strategy" {
        id: "9902c4e6-e6b7-4b11-9ed8-30c37c8e9426"
        performed_by: "be0d3f06-3e9b-4162-9ac6-2b16663d1dfa"
    }
    operational_activity "Make Priority Audio Announcement" {
        id: "41525b3e-aa1f-4002-b245-5e6c0a129fc0"
        performed_by: "c4fd192c-6826-4924-bbc3-3a4286b658b4"
    }
    operational_activity "Adapt Entertainment Service" {
        id: "02cf746c-0158-41cf-b626-e3a631132c1e"
        summary: "Adapt Entertainment Service to Flying Phases and Conditions"
        performed_by: "dab8526e-cf3a-4c8a-8fa1-7df4aa281bcf"
    }
    operational_activity "Broadcast Movies" {
        id: "7f5d8bbc-eca5-4656-a365-757543a38ab3"
        performed_by: "dab8526e-cf3a-4c8a-8fa1-7df4aa281bcf"
    }
    operational_activity "Play Imposed Movie" {
        id: "7915575a-916b-49ed-a5f0-c782e9135d68"
        performed_by: "87bfb1b6-b85e-4497-950d-663dd78c4502"
    }
    operational_activity "Make Audio Announcement" {
        id: "280bb458-fb27-41b5-84cf-4af9dd82f2a8"
        description: "This service allows to \"impose\" an audio annoucement (ie. uch as boarding, door closure, take off, turbulence, and landing announcements, ...). This service has priority over Entertainement Services."
        performed_by: "87bfb1b6-b85e-4497-950d-663dd78c4502"
    }
    operational_activity "Broadcast Live Moving-Map Channel" {
        id: "f15d0666-1b4a-48d3-a2b1-4206853f1d5a"
        performed_by: "dab8526e-cf3a-4c8a-8fa1-7df4aa281bcf"
    }
    operational_activity "Set Flying Mode" {
        id: "01a4afcf-b2ef-4562-aa09-6f6fda9f0b93"
        performed_by: "c4fd192c-6826-4924-bbc3-3a4286b658b4"
    }
    operational_activity "Update Digital Media Offer" {
        id: "7bd0bcf4-2fcc-4229-81f2-8d5cdac49ac8"
        performed_by: "be0d3f06-3e9b-4162-9ac6-2b16663d1dfa"
    }
    operational_activity "Deploy New Digital Media" {
        id: "1cc1517f-4971-4fbd-be1c-5c83631319a5"
        performed_by: "d7a95c22-1622-41d5-8201-37cd463075c7"
    }
    operational_activity "Connect to Wifi Network" {
        id: "4d274ffe-5010-4470-992c-cea2ae1587c8"
        performed_by: "dccf58d4-fe45-4a13-bb91-dc1b0c1cc0f5"
    }
    operational_activity "Provide Internet Access" {
        id: "dbe67e60-fb6b-4bd1-a85c-039b333e5b0b"
        performed_by: "dccf58d4-fe45-4a13-bb91-dc1b0c1cc0f5"
    }
    operational_activity "Provide Connectivity" {
        id: "d1d4b34b-685b-4c37-8db4-1d69bb16eb63"
        performed_by: "dab8526e-cf3a-4c8a-8fa1-7df4aa281bcf"
    }
    operational_activity "Play Ambiance Music" {
        id: "f4596943-e11d-425c-8b41-4ae491efd471"
        performed_by: "87bfb1b6-b85e-4497-950d-663dd78c4502"
    }
    operational_activity "Broadcast Audio" {
        id: "cc39bfa2-731b-448d-a8b7-a45d1cf45a97"
        performed_by: "dab8526e-cf3a-4c8a-8fa1-7df4aa281bcf"
    }
    operational_activity "Configure Availability Of Services" {
        id: "79a50368-c014-4410-aebd-627426b62614"
        performed_by: "d7a95c22-1622-41d5-8201-37cd463075c7"
    }
    interaction "Aircraft Position" { id: "6e25e03b-232a-4e36-89cc-26417caba9e3" from: "426f80c2-78ad-4e94-8e5b-da19f22aca26" to: "f15d0666-1b4a-48d3-a2b1-4206853f1d5a" }
    interaction "Audio Announcement" { id: "daaa759b-bfa4-49bb-89a6-08d99b603af9" from: "280bb458-fb27-41b5-84cf-4af9dd82f2a8" to: "cc39bfa2-731b-448d-a8b7-a45d1cf45a97" }
    interaction "Priority Audio Announcement" { id: "c3b68964-43f6-4150-990e-33a3cf7e9f6d" from: "41525b3e-aa1f-4002-b245-5e6c0a129fc0" to: "cc39bfa2-731b-448d-a8b7-a45d1cf45a97" }
    interaction "Service Availability Schemes" { id: "aa8e553d-88da-4f1d-896c-5ec63d1d8a76" from: "9902c4e6-e6b7-4b11-9ed8-30c37c8e9426" to: "79a50368-c014-4410-aebd-627426b62614" }
    interaction "Aircraft Direction" { id: "0bfe09db-1877-4dab-beeb-6c74baa8a951" from: "426f80c2-78ad-4e94-8e5b-da19f22aca26" to: "f15d0666-1b4a-48d3-a2b1-4206853f1d5a" }
    interaction "Aircraft Altitude" { id: "bb7ab210-a8e8-4c98-8fe1-a66a212ac6a3" from: "426f80c2-78ad-4e94-8e5b-da19f22aca26" to: "f15d0666-1b4a-48d3-a2b1-4206853f1d5a" }
    interaction "Services Configuration" { id: "6544013a-64f7-4411-a669-9876cea2cc47" from: "79a50368-c014-4410-aebd-627426b62614" to: "02cf746c-0158-41cf-b626-e3a631132c1e" }
    interaction "Digital Media Offer" { id: "ed894222-0666-4f33-9402-c388b424383f" from: "7bd0bcf4-2fcc-4229-81f2-8d5cdac49ac8" to: "1cc1517f-4971-4fbd-be1c-5c83631319a5" }
    interaction "Internet Access" { id: "39e4a609-fb84-4179-9c34-0803f3e09d0f" from: "dbe67e60-fb6b-4bd1-a85c-039b333e5b0b" to: "d26f381c-470d-4e9f-a868-cca9bd4a76f6" }
    interaction "Wifi Access" { id: "210a91cd-379a-450e-87eb-b996b4cd8c55" from: "4d274ffe-5010-4470-992c-cea2ae1587c8" to: "dbe67e60-fb6b-4bd1-a85c-039b333e5b0b" }
    interaction "Connectivity" { id: "959ac059-3465-4d28-99fa-30463fcec140" from: "d1d4b34b-685b-4c37-8db4-1d69bb16eb63" to: "4d274ffe-5010-4470-992c-cea2ae1587c8" }
    interaction "Imposed Movie" { id: "c80d7d2b-61d4-4c72-aaca-7055a6949c67" from: "7f5d8bbc-eca5-4656-a365-757543a38ab3" to: "f9cabb24-100b-4e68-930a-8f35ec38c106" }
    interaction "Safety Instructions" { id: "b55b0fd7-f59c-4976-aab1-2f19493cafde" from: "7915575a-916b-49ed-a5f0-c782e9135d68" to: "7f5d8bbc-eca5-4656-a365-757543a38ab3" }
    interaction "Commercial Ads" { id: "b4884221-ea38-4ab1-a58f-23a3bd57ea46" from: "7915575a-916b-49ed-a5f0-c782e9135d68" to: "7f5d8bbc-eca5-4656-a365-757543a38ab3" }
    interaction "Ambiance Music" { id: "3b814835-c766-48ff-819a-379921f146da" from: "cc39bfa2-731b-448d-a8b7-a45d1cf45a97" to: "60dc5173-5420-44c0-bedf-ad3f6958a79d" }
    interaction "Chosen Music" { id: "195cfecc-4cf8-43e4-85ab-6cc73311e388" from: "cc39bfa2-731b-448d-a8b7-a45d1cf45a97" to: "60dc5173-5420-44c0-bedf-ad3f6958a79d" }
    interaction "Moving Map" { id: "f21c4d30-f56c-412b-b7a7-1e072429076c" from: "f15d0666-1b4a-48d3-a2b1-4206853f1d5a" to: "45857975-71b5-4d8c-be67-0edb138873d0" }
    interaction "Ambiance Music (2)" { id: "dec8a64b-2308-4e54-8baa-49e188b7b727" from: "f4596943-e11d-425c-8b41-4ae491efd471" to: "cc39bfa2-731b-448d-a8b7-a45d1cf45a97" }
    interaction "Chosen Movie" { id: "b5cf7709-6acd-4494-b28a-08f794b843ce" from: "7f5d8bbc-eca5-4656-a365-757543a38ab3" to: "f9cabb24-100b-4e68-930a-8f35ec38c106" }
    interaction "Music Selection" { id: "b7d8100d-404a-4b7b-b08b-01f232cd0335" from: "60dc5173-5420-44c0-bedf-ad3f6958a79d" to: "cc39bfa2-731b-448d-a8b7-a45d1cf45a97" }
    interaction "Movie Selection" { id: "3d34fde4-8b91-4602-be32-733f3adcc472" from: "f9cabb24-100b-4e68-930a-8f35ec38c106" to: "7f5d8bbc-eca5-4656-a365-757543a38ab3" }
    interaction "Flying Mode" { id: "2272b681-dc35-4749-a2ad-421327e80abc" from: "01a4afcf-b2ef-4562-aa09-6f6fda9f0b93" to: "02cf746c-0158-41cf-b626-e3a631132c1e" }
    interaction "Audio Announcement (2)" { id: "d3db593e-dfeb-4ecf-b7cd-92b43ae785e0" from: "cc39bfa2-731b-448d-a8b7-a45d1cf45a97" to: "60dc5173-5420-44c0-bedf-ad3f6958a79d" }
    communication_means "Crew-Aircraft Digital Network" { id: "b107c6d4-d746-46cf-92f9-4ad7cae7940f" from: "e1b91d3f-beb2-40fd-869a-ad45ab8ec951" to: "dab8526e-cf3a-4c8a-8fa1-7df4aa281bcf" }
    communication_means "Maintenance-Aircraft\n Digital Network" { id: "00ad22c7-6458-40e1-91f2-8534ef638cb9" from: "d7a95c22-1622-41d5-8201-37cd463075c7" to: "dab8526e-cf3a-4c8a-8fa1-7df4aa281bcf" }
    operational_process "Broadcast Safety Instructions Movie" {
        id: "e5c28cc3-d846-47fe-98a2-0ccdb2ee512e"
        involves: ["7915575a-916b-49ed-a5f0-c782e9135d68", "7f5d8bbc-eca5-4656-a365-757543a38ab3", "f9cabb24-100b-4e68-930a-8f35ec38c106"]
    }
    operational_process "Perform Audio Announcement" {
        id: "c27f6524-d09e-4962-8f71-c347796bfe6f"
        involves: ["280bb458-fb27-41b5-84cf-4af9dd82f2a8", "cc39bfa2-731b-448d-a8b7-a45d1cf45a97", "60dc5173-5420-44c0-bedf-ad3f6958a79d"]
    }
    operational_capability "Entertain During Flight" {
        id: "2569c843-e368-451c-b7be-67af191a35a3"
        involves: ["dab8526e-cf3a-4c8a-8fa1-7df4aa281bcf", "2060479f-e59f-4b8f-975f-f05f5986e9b3", "87bfb1b6-b85e-4497-950d-663dd78c4502"]
    }
    operational_capability "Perform Flight On-Board Announcements" {
        id: "2ce82a6f-0467-41a5-9c06-28f14ef1dd1b"
        involves: ["87bfb1b6-b85e-4497-950d-663dd78c4502", "c4fd192c-6826-4924-bbc3-3a4286b658b4"]
    }
    operational_capability "Implement a Commercial Strategy" {
        id: "2ced3ae3-205b-4c3b-95dd-ee36361dbe9b"
        involves: ["be0d3f06-3e9b-4162-9ac6-2b16663d1dfa", "7bd0bcf4-2fcc-4229-81f2-8d5cdac49ac8", "9902c4e6-e6b7-4b11-9ed8-30c37c8e9426"]
    }
}

system_analysis "System Analysis" {
    id: "1d30b780-f222-4794-9f78-5b70ce799bf0"
    function "Perform Cabin Management Activities" {
        id: "3398fdbf-3415-4d03-a457-477324d2e091"
        function "Play Airline-Imposed Videos" {
            id: "cfde8bb1-4df9-4573-a516-41df984ac753"
            description: "This function consists in triggering the play of pre-recorded airline videos. In particular, these imposed videos inclure pre-flight safety instructions (detailed explanation given before takeoff to airline passengers about the safety features of a commercial aircraft). Such imposed videos can also be commercial ads."
            port in "FIP1" { id: "bd1fed2c-d331-4b92-9eed-ff5db97debff" }
            port in "FIP2" { id: "c9d13362-83c6-40d3-a7e5-10b7222a7cd9" }
            port out "FOP1" { id: "da43a1c4-67b6-4082-971b-4efe8559539c" }
            port out "FOP2" { id: "e0654cac-e572-4ad1-a077-297f7fbf499a" }
            port out "FOP 1" { id: "76e1c0e5-3567-4094-b84d-779b35388d9a" }
            port out "FOP 2" { id: "9c0803a1-79f4-4d1d-a545-100f5b4bccc3" }
        }
        function "Launch Pre-Flight Tests" {
            id: "64847aed-d627-4812-a2de-a5dd8d7ce573"
            description: "This function consists in triggering a series of system auto-tests before each flight."
        }
    }
    function "Entertain with IFE System" {
        id: "3688cf59-97ee-4446-8041-7941cee6c767"
        function "Watch Video on Private Screen" {
            id: "880520c2-6bd9-4481-b9da-6ccee7b9d665"
            description: "This function represents the fact that the passenger can watch a movie/video on a private screen (i.e. the in-seat TV)."
            port in "FIP1" { id: "4140cda0-39e5-42fc-a312-e72bf83eec77" }
        }
        function "Watch Video on Cabin Screen" {
            id: "1bc70a69-c6ad-4203-abb9-88b644ac0b95"
            description: "This function represents the fact that the passenger can watch a video on a cabin screen (typically located in the ceiling)."
            port in "FIP3" { id: "fe8c82bd-8c8f-48bd-8541-4c1e4d57a199" }
            port in "FIP2" { id: "e5700496-23ed-4837-8b94-382fcca719ee" }
        }
        function "Select Passenger Service" {
            id: "65787ee3-ab55-4200-91e3-d772a4ee3f28"
            description: "This function covers the selection by the passenger of a given IFE service (gaming, moving-map, VOD, etc.)."
            port in "FIP1" { id: "4044bcd5-a02f-48c0-8425-41969f57ec7f" }
            port out "FOP1" { id: "841c77e2-381e-4594-a0a3-96add7a5f55e" }
        }
        function "Command VOD Service" {
            id: "4dbd5747-9662-4378-88cf-270b69ef3827"
            description: "This function is dedicated to the interactions in the context of the VOD service: Selection of movies and control on these movies (play, pause, stop, etc.)."
            port in "FIP 2" { id: "4ccf7302-dae8-4e61-bb53-336ec6b8a6aa" }
            port in "FIP 1" { id: "81e69d4d-36cf-4b98-a61d-806d2bdb384e" }
            port in "FIP 3" { id: "c8671941-47ef-49dc-92b9-d55234f2fb31" }
            port out "FOP 2" { id: "13cf5997-3a3b-42bf-a697-c3e24e7fbbe6" }
            port out "FOP 1" { id: "68bd51f1-1da0-4eae-b4ef-5e306a9c1edc" }
            port out "FOP 5" { id: "753b3eb0-f4ae-4c0a-aca6-cc1b542a6516" }
            port out "FOP 6" { id: "d0a749de-3ab1-4881-ba99-e135c7e62d5f" }
            port out "FOP 3" { id: "bfdaa97b-16fd-4673-97da-faac3c6e7b19" }
        }
        function "Listen to Audio Announcement" {
            id: "e04025f9-6b52-4377-bac1-78d0bf2f8fed"
            description: "Audio announcements are information about the flight provided by the cabin crew or the captain (boarding, pre-flight announcements, captain's announcements, safety briefing, turbulence, take-off/ascent, descent/final, landing, etc.). The passenger can listen to these annoucements aither through the aircraft speakers either through the individual Seat TVs."
            port in "FIP 1" { id: "75b87bc9-d4d1-4bf9-9a76-4fade39ebf32" }
            port in "FIP 2" { id: "e0522dc7-4350-4299-b398-fa415a948976" }
        }
    }
    function "Provide Aircraft Information, Commands and Means" {
        id: "7de3b04b-6291-4098-afbd-2fb72cd25022"
        function "Provide Satellite Communication Means" {
            id: "82ab0ab9-2c03-4689-989a-530aae6fcf94"
            description: "This function consists in providing satellite network, typically for telephony or in-cabin wifi systems."
        }
        function "Provide Navigation Data" {
            id: "6ce8f26a-ef90-4420-94e1-acf6b2393720"
            description: "Navigation is the determination of position and direction on or above the surface of the Earth. Avionics can use satellite-based systems (such as GPS and WAAS), ground-based systems (such as VOR or LORAN), or any combination thereof. The IFE system requires this data to run the moving-map service."
        }
        function "Provide Electrical Power" {
            id: "135e77a7-79ef-45e8-b138-67e283a14cdd"
            description: "This function is responsible for providing power consumption profiles. According to different flying conditions and phases, the power made available to the IFE system may differ."
            port out "FOP 1" { id: "e12ce047-aae2-4904-bd89-8e20c17f7651" }
        }
        function "Provide Exterior-View Videos" {
            id: "dacbebb6-72c7-4d4c-9009-aa95eba16dd3"
            description: "This functions relays the videos captured external cameras, typically located on the tail or the nose of the plane."
        }
        function "Send Audio Announcement" {
            id: "8f279b07-26aa-46f6-8c9a-2a132f0b1177"
            description: "Audio announcement include: boarding, pre-flight announcements, captain's announcements, safety briefing, turbulence, take-off/ascent, descent/final, landing, etc."
            port out "FOP 5" { id: "8ea7d204-dd52-4e80-b384-f4de886461c1" }
        }
        function "Play Sound in Cabin" {
            id: "6b9e59a9-89f2-497e-a348-3aef80689e61"
            description: "This function receives digital audio streams and play them on the aricraft speakers."
            port in "FIP 1" { id: "e3f96e27-3878-4461-81af-874cb55f0e3f" }
            port out "FOP1" { id: "5cb257a9-5a0b-49dd-8fea-778c1f8d999b" }
        }
        function "Send Decompression Notification" {
            id: "9972b7a8-df8a-4626-b298-1dd0d19e451e"
            description: "If ever a decompression occurs, a notification is sent to the IFE system (which will automatically swtich to \"Halted\" mode."
            port out "FOP 1" { id: "c4122b7e-b3d5-48ad-a61e-4cbdd25c5693" }
        }
    }
    function "Perform Maintenance, Configuration and Test Operations" {
        id: "cd62e8fb-8a89-4f33-bd0f-a2bb935a5e08"
        function "Update Media Content" {
            id: "4d0046b4-4865-438d-b09b-fd95671db09b"
            description: "This function consists in updating the IFE media server with new content. Using a specific device, the ground operator can for example updload new movies/music and remove others."
            port out "FOP 1" { id: "c579a89a-99f8-4182-bcb6-da6193c3a682" }
        }
        function "Configure the System" {
            id: "88faafec-6848-4494-8dd3-2acda3e53688"
            description: "[Not coevered in this version of the model]"
        }
        function "Launch and Analyze Tests" {
            id: "ff70c856-f667-4e61-98d7-00190703d53a"
            description: "[Not covered in the version of the model] This function consists in triggering system tests, typically during maintenance phases."
        }
        function "Set Passenger Service Authorization" {
            id: "5e0fff47-a1a5-4a13-867c-18348bd894af"
            description: "The airline company may have specific business strategies: According to the class they are travelling in, passengers may benefit from different IFE services. This function consists in uploading a profile that describes the authorization schemes."
            port out "FOP 1" { id: "5fbee0e5-cba5-47c2-84f5-d5050c91ccdd" }
        }
    }
    function "IFE System" {
        id: "6d10ccfa-1be8-4ab6-a0d8-74941051595b"
        function "Provide Access to Digital Media" {
            id: "6b54b12a-50f8-4ce5-b935-08edbf655d1f"
            function "Store Digital Media" {
                id: "d683afa5-907e-494d-8f35-e10db839feaf"
                description: "This function is responsible for the storage of audio and video files that will be streamed and of movie information such as metadata (pictures,description)."
                port in "FIP 1" { id: "41706e91-df75-4ae4-b6a7-c6f3aea01325" }
                port out "FOP 1" { id: "aff54b9f-8e07-4837-965e-40802c6a3c86" }
                port out "FOP 2" { id: "0db6dc4a-07e3-4739-9f41-3d920f8c9a1f" }
                port out "FOP 3" { id: "5a47aa37-e709-4ad9-94d4-d37b6adcfc8b" }
                port out "FOP 4" { id: "8f3bc1a6-d7c3-4ce5-ac9b-5863f632ad89" }
                port out "FOP 5" { id: "86502274-bc12-47bb-b0db-366421844017" }
                port out "FOP 6" { id: "ed1b8881-5247-497e-a05f-26fd58edb4c0" }
            }
            function "Load Digital Media" {
                id: "a16d1d72-9db2-4554-af57-411b29793904"
                description: "This function is responsible for receiving the media content uploaded by the ground operator and for moving it onto the IFE media storage unit."
                port in "FIP 1" { id: "e6e908ef-37ba-4001-b32f-362a9629f3b2" }
                port out "FOP 1" { id: "8204bf11-857f-41a2-8322-76f507e66926" }
            }
        }
        function "Manage Passenger Services Lifecycle" {
            id: "8a4b7443-4f73-49c2-a175-940a5d8accea"
            description: "This function is responsible for controling the launch of services such as VOD, gaming, moving-map, etc. It determines the availability of the services given the airline company authorization scheme and the constraints received from the aircraft. This function provides means to select among a predefined list of services."
            function "Determine Service Availability" {
                id: "b7244d6a-08ab-4455-bbd9-aa5f1bbc9b5b"
                description: "This function is responsible for computing the availability of passenger services based on the authorization scheme (airline business strategy) and the aircraft contraints."
                available_in_states: ["b2c226dc-43c6-494e-b6cb-dbc652561725", "6c60fb24-82ff-482a-87db-58f00e0323f4", "75503038-7ad6-4212-9098-b5764b7a430b", "6171dcbd-85b7-4452-a035-8ab9c1be37e7"]
                port in "FIP 1" { id: "fecdd587-c206-485f-b500-3972e89636ac" }
                port in "FIP 2" { id: "2c161e05-569b-45cd-a4cc-c9eca32a570d" }
                port out "FOP 1" { id: "7fdc1a1d-fdc3-45f9-bd24-8daf808cae98" }
            }
            function "Handle Service Activation Requests" {
                id: "2c453928-a098-4f8a-a1e6-fc545a2840d1"
                description: "This function displays the available services and trigger service activations when applicable."
                port in "FIP1" { id: "ee142c62-9963-4665-bb40-2bb282ea2263" }
                port in "FIP 4" { id: "d720121f-6907-4266-8af7-7ae5928e01f8" }
                port out "FOP1" { id: "957f0421-6978-4fe0-a4e8-01ae172f783e" }
                port out "FOP 4" { id: "24a53b80-4400-4a33-9ca3-bd6b7b8657bd" }
            }
        }
        function "Run Services" {
            id: "0bfc000c-dd38-40d3-9cd3-2037905a7ab3"
            description: "This function regroups all IFE services running functions."
            function "Run Video-On-Demand Service" {
                id: "26cd71b0-ec92-41a9-b9fb-6878beede810"
                description: "VOD Service is a passenger service. This function gathers all functions in charge of managing the VOD-related interactions of the passenger (selection of movies, play/pause/stop controls, display of movie data, etc.)."
                function "Handle VOD Movie Controls" {
                    id: "5e221dee-8dc7-4a29-9224-ab53fedb3995"
                    description: "This function forwards the VOD Movie play/pause/stop commands."
                    available_in_states: ["6171dcbd-85b7-4452-a035-8ab9c1be37e7"]
                    port in "FIP 1" { id: "9d559cc6-746c-4429-9eb9-415ce19ec8f6" }
                    port in "FIP 2" { id: "d86cb35f-1c85-4c3e-a2fa-fcfd7bfef9fa" }
                    port in "FIP 3" { id: "78a2b2fb-d146-42f1-a7e4-94d64f01d8fe" }
                    port out "FOP 2" { id: "70cf1f55-3d49-4817-b780-f92f72316f97" }
                }
                function "Retrieve VOD Movie Data" {
                    id: "0a2099f5-2831-4beb-89ae-996a6f54d032"
                    description: "This function consists both in displaying lists of available VOD movies and capturing the passenger VOD selections."
                    port in "FIP 1" { id: "ada746c6-371f-4478-aa68-06a0e9d281e9" }
                    port in "FIP 2" { id: "e2366cc3-835e-45d2-89cf-c82f93edd26f" }
                    port in "FIP 3" { id: "b7b76a14-6bb6-45a5-8d27-94fdaca71bed" }
                    port in "FIP 4" { id: "41b33ff4-b487-4311-b1b7-26184d766070" }
                    port in "FIP 5" { id: "fd4d4602-b133-4f72-a72e-7025d5eae330" }
                    port in "FIP 6" { id: "95c1bc62-2714-4cd1-9ff1-74351e11667f" }
                    port in "FIP 7" { id: "1f811239-3dce-4585-b0fa-62878c31fba1" }
                    port out "FOP 2" { id: "26c4a754-d576-46ce-812c-622b5de544bf" }
                    port out "FOP 1" { id: "434e8728-2fd9-43d3-9413-8007c8a026b4" }
                    port out "FOP 3" { id: "042c3342-a428-4d59-996e-7c0d3f729f1a" }
                }
            }
            function "Run Cabin Intercommunication Service" {
                id: "7b7b0d2c-27e8-4ca0-8a5f-2790a37e97b6"
                description: "Cabin intercommunication is a cabin crew service. This function gathers all functions in charge of managing the cabin crew interactions related to imposed videos (selection of videos, play/pause/stop controls, display of video data, etc.)."
                function "Handle Imposed Videos Controls" {
                    id: "06b8dd9b-2199-4ad1-a9b2-229d1de1bb48"
                    description: "This function forwards the imposed videos play/pause/stop commands."
                    port in "FIP 4" { id: "289c57c3-4817-4415-b497-0ab8f38ea658" }
                    port in "FIP 5" { id: "8d242438-0eaa-4b1c-8988-39bbe145247b" }
                    port in "FIP3" { id: "482fd2b4-c5a6-4237-bf18-da83d0faeaac" }
                    port out "FOP 3" { id: "38aab4df-265a-4f66-8177-fac28b42c1f6" }
                }
                function "Manage Imposed Video Selections" {
                    id: "6e577fe5-ebd7-409e-b5de-b876fd689851"
                    description: "This function consists both in displaying lists of available imposed videos and capturing the cabin crew selections."
                    port in "FIP3" { id: "8714bcd3-4245-404a-b610-c08d8acde400" }
                    port in "FIP1" { id: "c16ff8c0-05ae-4591-82a8-478aaf6ec043" }
                    port out "FOP1" { id: "6b424ace-11b0-48bc-ba7c-bcafc5f9aab5" }
                }
            }
            function "Run Gaming Service" {
                id: "00cc4e40-09b1-42c5-b945-32743e1c9968"
                description: "Gaming Service is a passenger service. [Not yet covered in this model]."
            }
            function "Run Any Other Service" {
                id: "f44bd6aa-c54a-49d1-8010-e96dab3ad00a"
                description: "[Not yet covered in this model]."
            }
            function "Run Airline Channel Service" {
                id: "4d8cc764-6c72-49ca-8b5e-c57104345aac"
                description: "Airline Channel is a cabin crew service. [Not yet covered in this model]."
            }
        }
        function "Manage Video and Audio Diffusion" {
            id: "ba4d638d-e6bb-4908-94f4-eb974405c733"
            description: "This function is responsible for broadcasting audio and video streams, should they be VOD movies, imposed videos, on-demand or imposed music. This function also covers the transmission of audio announcements (coming from the aircraft) and the IFE passenger service interruptions triggered by these announcements."
            function "Broadcast Audio Video Streams" {
                id: "e1808524-dfa9-4883-a809-c44a87712d1b"
                description: "This function is responsible for broadcasting existing audio and video media as well as transmitting audio announcements coming from the aircraft. Transmitting audio and video streams means: - sending them to each passenger (on Seat TVs) - sending them to the cabin screens - providing the cabin crew with a status of their running imposed videos."
                status: "TODO"
                review: "dezd"
                port in "FIP 1" { id: "d5a90858-f66d-4643-9a63-2dca7f5c5bac" }
                port in "FIP 2" { id: "1da4d4b3-8f6d-45ea-a82a-e493b9b5113c" }
                port in "FIP 3" { id: "fb4118c7-9bc6-4e7c-aedb-15d947b348ef" }
                port in "FIP 4" { id: "0bf85ea8-1d8f-43b3-8c8a-dd04d921eb6f" }
                port in "FIP 5" { id: "77d1c4fe-d08b-404b-833b-d306a02dc6cf" }
                port out "FOP 1" { id: "6e7508dd-ef4d-4419-92bb-c2396201bfdb" }
                port out "FOP 2" { id: "cd2b1637-f246-4eb1-ad60-b468c961c97e" }
                port out "FOP 3" { id: "c3d87ee2-2493-47f9-a492-fa80bd355555" }
                port out "FOP 4" { id: "5c426ed8-e3b9-46b5-a715-01e91c821b92" }
                port out "FOP 5" { id: "b9ace19f-2d94-412d-ae1e-34b6fe329b6f" }
                port out "FOP 6" { id: "1a86cf04-6b46-47af-a28c-efcaf981ff58" }
            }
            function "Manage Passenger Service Interruptions" {
                id: "d8b8c1bd-a2ea-47fd-a7a9-0239408db1a5"
                description: "This function is responsible for triggering passenger service interruptions as soon as an imposed video is played or an audio announcement is performed. When a VOD movie is running, this means for example that the movie as to be automatically paused and the current position stored."
                available_in_states: ["6171dcbd-85b7-4452-a035-8ab9c1be37e7"]
                port in "FIP 3" { id: "bdb71b4e-fb16-4af0-87ca-03775716f71c" }
                port out "FOP 3" { id: "4e21b270-bd09-4fcd-87c8-b72eb8c3825f" }
            }
        }
        function "Determine Operating Profiles" {
            id: "3f8567f2-5389-42c0-84a9-8a1360dc9572"
            description: "Based on the notifications it receives from the aircraft, this function computes the mode in which the IFE system has to operate. For example, if the aircraft sends a power reduction request, the IFE may switch to a downgraded mode."
            port in "FIP 1" { id: "9f9c4a57-f611-4a37-8af8-2bc1d009735a" }
            port in "FIP 2" { id: "a22ec024-5e3e-452a-a61d-9cbd6d6d06ca" }
            port out "FOP 1" { id: "05fd95ce-873c-4772-8156-739a6383ce86" }
        }
    }
    functional_exchange "Digital Media" { id: "56fc8021-e30e-44ae-9d5e-f776874ea5a9" from: "a16d1d72-9db2-4554-af57-411b29793904.FOP 1" to: "d683afa5-907e-494d-8f35-e10db839feaf.FIP 1" }
    functional_exchange "Service Availability" { id: "6dda546d-7b7f-4ff4-939c-d9c8e441a3a8" from: "b7244d6a-08ab-4455-bbd9-aa5f1bbc9b5b.FOP 1" to: "2c453928-a098-4f8a-a1e6-fc545a2840d1.FIP 4" }
    functional_exchange "VOD Movie Status" { id: "19cd3b56-9087-4ee6-bf0b-ebd2f730a3e3" from: "d8b8c1bd-a2ea-47fd-a7a9-0239408db1a5.FOP 3" to: "e1808524-dfa9-4883-a809-c44a87712d1b.FIP 4" }
    functional_exchange "VOD Movie" { id: "2d2e8936-7dc3-40da-bb09-6d3f2ce71921" from: "e1808524-dfa9-4883-a809-c44a87712d1b.FOP 6" to: "d8b8c1bd-a2ea-47fd-a7a9-0239408db1a5.FIP 3" }
    functional_exchange "Available VOD Movies List" { id: "5e30c0be-9342-477c-82a1-c1d1ba0682fa" from: "d683afa5-907e-494d-8f35-e10db839feaf.FOP 3" to: "0a2099f5-2831-4beb-89ae-996a6f54d032.FIP 3" category: "Media Content" }
    functional_exchange "Displayed VOD Movies List" { id: "8dcadf53-92b7-453c-a4a6-3e3780eabb82" from: "0a2099f5-2831-4beb-89ae-996a6f54d032.FOP 3" to: "4dbd5747-9662-4378-88cf-270b69ef3827.FIP 1" category: "Displayed VOD Movie Data" }
    functional_exchange "Available Languages" { id: "13425779-3b59-4d68-8734-cd9717b69d29" from: "d683afa5-907e-494d-8f35-e10db839feaf.FOP 4" to: "0a2099f5-2831-4beb-89ae-996a6f54d032.FIP 2" category: "Media Content" }
    functional_exchange "Movie Cover" { id: "018a09b4-bb7a-4714-8814-613e3af68bfe" from: "d683afa5-907e-494d-8f35-e10db839feaf.FOP 6" to: "0a2099f5-2831-4beb-89ae-996a6f54d032.FIP 4" category: "Media Content" }
    functional_exchange "Displayed Available Languages" { id: "c8bfbb5b-865d-4924-902f-b209dbfb3433" from: "0a2099f5-2831-4beb-89ae-996a6f54d032.FOP 2" to: "4dbd5747-9662-4378-88cf-270b69ef3827.FIP 2" category: "Displayed VOD Movie Data" }
    functional_exchange "Displayed Movie Description" { id: "728283a8-142b-4afe-b67c-f2f75cf5a3a3" from: "0a2099f5-2831-4beb-89ae-996a6f54d032.FOP 1" to: "4dbd5747-9662-4378-88cf-270b69ef3827.FIP 3" category: "Displayed VOD Movie Data" }
    functional_exchange "Available Airline Video List" { id: "4411a2bf-9f4e-40f3-b66e-c82b89483b66" from: "d683afa5-907e-494d-8f35-e10db839feaf.FOP 2" to: "6e577fe5-ebd7-409e-b5de-b876fd689851.FIP1" category: "Media Content" }
    functional_exchange "Displayed Imposed Videos List" { id: "21e787ec-7763-4896-a0c6-421367c3561a" from: "6e577fe5-ebd7-409e-b5de-b876fd689851.FOP1" to: "cfde8bb1-4df9-4573-a516-41df984ac753.FIP1" category: "Displayed Imposed Video Data" }
    functional_exchange "Played Audio Video" { id: "15c3b494-a0b8-4aca-b734-01978120cbd4" from: "e1808524-dfa9-4883-a809-c44a87712d1b.FOP 1" to: "880520c2-6bd9-4481-b9da-6ccee7b9d665.FIP1" }
    functional_exchange "Passenger VOD Service Selection" { id: "af919966-017f-44fa-856a-f680d604ded8" from: "65787ee3-ab55-4200-91e3-d772a4ee3f28.FOP1" to: "2c453928-a098-4f8a-a1e6-fc545a2840d1.FIP1" }
    functional_exchange "Pause Command" { id: "b40bc1cf-4622-41ec-801f-1f37102f7ff9" from: "cfde8bb1-4df9-4573-a516-41df984ac753.FOP2" to: "06b8dd9b-2199-4ad1-a9b2-229d1de1bb48.FIP3" category: "Imposed Video Control Commands" }
    functional_exchange "VOD Movie Play Selection" { id: "0ce48a3f-6a6d-498c-b41e-280d1bc64928" from: "4dbd5747-9662-4378-88cf-270b69ef3827.FOP 2" to: "5e221dee-8dc7-4a29-9224-ab53fedb3995.FIP 1" category: "VOD Movie Control Commands" }
    functional_exchange "Audio Video Media" { id: "dc5045e3-96c3-4fc1-b144-305a23255b35" from: "d683afa5-907e-494d-8f35-e10db839feaf.FOP 1" to: "e1808524-dfa9-4883-a809-c44a87712d1b.FIP 1" }
    functional_exchange "Audio Signal" { id: "5a141688-cd8a-4211-84ad-0d55c85888cd" from: "6b9e59a9-89f2-497e-a348-3aef80689e61.FOP1" to: "1bc70a69-c6ad-4203-abb9-88b644ac0b95.FIP3" }
    functional_exchange "Audio Stream" { id: "5d23ddc8-b208-4db0-8b20-680568afdc92" from: "e1808524-dfa9-4883-a809-c44a87712d1b.FOP 3" to: "6b9e59a9-89f2-497e-a348-3aef80689e61.FIP 1" }
    functional_exchange "Movie Description" { id: "323c5735-2b57-4f13-936f-edabf89c3785" from: "d683afa5-907e-494d-8f35-e10db839feaf.FOP 5" to: "0a2099f5-2831-4beb-89ae-996a6f54d032.FIP 1" category: "Media Content" }
    functional_exchange "VOD Movie Pause Selection" { id: "136ac12c-9e93-43b9-bc3d-4253ea48233a" from: "4dbd5747-9662-4378-88cf-270b69ef3827.FOP 5" to: "5e221dee-8dc7-4a29-9224-ab53fedb3995.FIP 3" category: "VOD Movie Control Commands" }
    functional_exchange "VOD Movie Stop Selection" { id: "7531be55-55d9-44a9-8077-727417cf4f0e" from: "4dbd5747-9662-4378-88cf-270b69ef3827.FOP 6" to: "5e221dee-8dc7-4a29-9224-ab53fedb3995.FIP 2" category: "VOD Movie Control Commands" }
    functional_exchange "Stop Command" { id: "0bc7709d-62f9-4a1e-af23-b5a8a78ac19c" from: "cfde8bb1-4df9-4573-a516-41df984ac753.FOP 1" to: "06b8dd9b-2199-4ad1-a9b2-229d1de1bb48.FIP 4" category: "Imposed Video Control Commands" }
    functional_exchange "Play Command" { id: "cc3abddf-d289-4461-925d-c4d962230954" from: "cfde8bb1-4df9-4573-a516-41df984ac753.FOP 2" to: "06b8dd9b-2199-4ad1-a9b2-229d1de1bb48.FIP 5" category: "Imposed Video Control Commands" }
    functional_exchange "Displayed Video" { id: "7f88eaab-0c1f-456b-b32f-f65c5bf00454" from: "e1808524-dfa9-4883-a809-c44a87712d1b.FOP 2" to: "1bc70a69-c6ad-4203-abb9-88b644ac0b95.FIP2" }
    functional_exchange "Imposed Video Playing Status" { id: "8aa8e222-d82c-4132-ac6d-9b5f8691a89e" from: "e1808524-dfa9-4883-a809-c44a87712d1b.FOP 4" to: "cfde8bb1-4df9-4573-a516-41df984ac753.FIP2" }
    functional_exchange "Imposed Video Control Command" { id: "0048b9e1-315c-4025-a694-d91e75250c19" from: "06b8dd9b-2199-4ad1-a9b2-229d1de1bb48.FOP 3" to: "e1808524-dfa9-4883-a809-c44a87712d1b.FIP 3" }
    functional_exchange "VOD Movie Control Command" { id: "4125bbcd-f124-4bd5-9f40-aa73c538a3e4" from: "5e221dee-8dc7-4a29-9224-ab53fedb3995.FOP 2" to: "e1808524-dfa9-4883-a809-c44a87712d1b.FIP 2" }
    functional_exchange "Passenger VOD Service Activation" { id: "e7ca817a-2207-4beb-b7ee-98a679f3e0fb" from: "2c453928-a098-4f8a-a1e6-fc545a2840d1.FOP 4" to: "0a2099f5-2831-4beb-89ae-996a6f54d032.FIP 7" }
    functional_exchange "Passenger Service Authorization Profile" { id: "99249234-8b24-490f-b746-383bfe3087b8" from: "5e0fff47-a1a5-4a13-867c-18348bd894af.FOP 1" to: "b7244d6a-08ab-4455-bbd9-aa5f1bbc9b5b.FIP 1" }
    functional_exchange "Displayed Available Passenger Services List" { id: "604081d7-e597-4441-b2e9-e002a9cb1ccd" from: "2c453928-a098-4f8a-a1e6-fc545a2840d1.FOP1" to: "65787ee3-ab55-4200-91e3-d772a4ee3f28.FIP1" }
    functional_exchange "VOD Movie Language Selection" { id: "fa5c887c-feac-438f-ac22-2bd908b30483" from: "4dbd5747-9662-4378-88cf-270b69ef3827.FOP 1" to: "0a2099f5-2831-4beb-89ae-996a6f54d032.FIP 6" category: "VOD Movie Selections" }
    functional_exchange "VOD Movie Selection" { id: "613d97e8-7adb-4f2b-a3de-359c73087bff" from: "4dbd5747-9662-4378-88cf-270b69ef3827.FOP 3" to: "0a2099f5-2831-4beb-89ae-996a6f54d032.FIP 5" category: "VOD Movie Selections" }
    functional_exchange "Imposed Video Selection" { id: "8016239b-3648-4d04-85d4-76305d740490" from: "cfde8bb1-4df9-4573-a516-41df984ac753.FOP1" to: "6e577fe5-ebd7-409e-b5de-b876fd689851.FIP3" }
    functional_exchange "Digital Media (2)" { id: "29fdf266-fe43-4715-b919-cb4ce94b35d7" from: "4d0046b4-4865-438d-b09b-fd95671db09b.FOP 1" to: "a16d1d72-9db2-4554-af57-411b29793904.FIP 1" }
    functional_exchange "Decompression Notification" { id: "49328560-baa6-492c-ad89-8166ce70fdad" from: "9972b7a8-df8a-4626-b298-1dd0d19e451e.FOP 1" to: "3f8567f2-5389-42c0-84a9-8a1360dc9572.FIP 1" }
    functional_exchange "Power Consomption Cap" { id: "149f7acb-773d-4960-93a6-df36efee81d0" from: "135e77a7-79ef-45e8-b138-67e283a14cdd.FOP 1" to: "3f8567f2-5389-42c0-84a9-8a1360dc9572.FIP 2" }
    functional_exchange "IFE Operating Profile" { id: "e3c0b2f5-ec50-4dba-9808-04d6179ad01e" from: "3f8567f2-5389-42c0-84a9-8a1360dc9572.FOP 1" to: "b7244d6a-08ab-4455-bbd9-aa5f1bbc9b5b.FIP 2" }
    functional_exchange "Audio Stream (2)" { id: "c9afe3ac-0a52-450a-b97d-741e4b03ad39" from: "8f279b07-26aa-46f6-8c9a-2a132f0b1177.FOP 5" to: "e1808524-dfa9-4883-a809-c44a87712d1b.FIP 5" }
    functional_exchange "Audio Signal (2)" { id: "c4d953c9-05e2-42ca-b110-7014f768d808" from: "6b9e59a9-89f2-497e-a348-3aef80689e61.FOP1" to: "e04025f9-6b52-4377-bac1-78d0bf2f8fed.FIP 1" }
    functional_exchange "Audio Signal (3)" { id: "883d204e-69ff-40e9-80e4-e4d5845bb2d2" from: "e1808524-dfa9-4883-a809-c44a87712d1b.FOP 5" to: "e04025f9-6b52-4377-bac1-78d0bf2f8fed.FIP 2" }
    system_component "IFE System" {
        id: "e0a46739-7e50-4ea6-b198-0e5efb9ce68f"
        description: "It is an overly simplified vision of what a real in-flight entertainment system is: IFE System described as a backbox, interacting with Actors: Passenger Cabin Crew Aircraft Ground Operator The system is globally organized as follows: Cabin crew services are always available, passenger services have to be activated. Their availability depends on the flying conditions and on the class the passengers are flying in The system is able to store digital media content Most of the interactions of the passengers with their services (navigating between menus, selecting movies, etc.) are captured in functions called &ldquo;Run &lt;xxx&gt; service&rdquo; Services rely on audio and video broadcast means Maintenance and configuration topics are only evoked The [MCB] All Missions and Capabilities diagram shows on the capabilities of the IFE system. Three other diagrams provide interesting entry points to the model: [SC] System Actors lists all the Actors (the aircraft is considered as an actor) [SDFB] Top Level Functional Overview provides a first functional overview of what the actors actually do and what kind of exchanges they have with the IFE system [SAB] Top Level System Overview and its slightly refined version [SAB] High Level System Overview are very good entry points to further navigate in the model. Starting from these high level diagrams, a typical reading path is to dive into the function to visualize their refinement. For example, [SDFB] [CTX] Run Services. And recursively. Using capabilities as a driver to browse the model is recommended. Some diagrams are specifically marked for that purpose, for instance [SDFB] [CAPABILITY] Provide Video Entertainment Services. From this capability covering VOD services to the passengers, browse the functional chains and scenarios to better understand how the capability is fulfilled. Functional chains: [SFCD] Start Playing VOD Movie, [SFCD] Resume VOD Movie Scenarios: [ES] Start VOD Service, [ES] Start Playing VOD Movie Check the global mode machine of the system with [MSM] IFE Operating Modes. Among others, this mode machine is intended to specify under which operational conditions the IFE system is running. Another Mode machine is specifically modelled for what will become the Seat TV: [MSM] Seat TV - Movie Player Modes Notes: Not all the IFE system is modelled. Focus is put on VOD service, audio announcement and imposed videos (safety instructions, ads, etc.). While interesting, the topic of the integration of the IFE with the aircraft is kept minimal (according to flying conditions, the IFE system is supposed to behave differently). Not all possible Scenarios and Functional Chains have been created."
        summary: "In-Flight Entertainment System"
        status: "TO BE DISCUSSED"
        review: "First proposal"
        allocated_functions: ["d8b8c1bd-a2ea-47fd-a7a9-0239408db1a5", "5e221dee-8dc7-4a29-9224-ab53fedb3995", "d683afa5-907e-494d-8f35-e10db839feaf", "2c453928-a098-4f8a-a1e6-fc545a2840d1", "b7244d6a-08ab-4455-bbd9-aa5f1bbc9b5b", "e1808524-dfa9-4883-a809-c44a87712d1b", "a16d1d72-9db2-4554-af57-411b29793904", "0a2099f5-2831-4beb-89ae-996a6f54d032", "6e577fe5-ebd7-409e-b5de-b876fd689851", "06b8dd9b-2199-4ad1-a9b2-229d1de1bb48", "3f8567f2-5389-42c0-84a9-8a1360dc9572", "00cc4e40-09b1-42c5-b945-32743e1c9968", "f44bd6aa-c54a-49d1-8010-e96dab3ad00a", "4d8cc764-6c72-49ca-8b5e-c57104345aac"]
    }
    actor "Passenger" {
        id: "181a678c-dca9-46c1-9d18-b5a0c457c0de"
        allocated_functions: ["65787ee3-ab55-4200-91e3-d772a4ee3f28", "880520c2-6bd9-4481-b9da-6ccee7b9d665", "1bc70a69-c6ad-4203-abb9-88b644ac0b95", "4dbd5747-9662-4378-88cf-270b69ef3827", "e04025f9-6b52-4377-bac1-78d0bf2f8fed"]
    }
    actor "Cabin Crew" {
        id: "4890eae8-7a62-4d7b-bf8a-1590a199eebf"
        allocated_functions: ["cfde8bb1-4df9-4573-a516-41df984ac753", "64847aed-d627-4812-a2de-a5dd8d7ce573"]
    }
    actor "Aircraft" {
        id: "c8b78c78-5b11-4fc0-87b7-3ca84622efea"
        allocated_functions: ["6b9e59a9-89f2-497e-a348-3aef80689e61", "82ab0ab9-2c03-4689-989a-530aae6fcf94", "6ce8f26a-ef90-4420-94e1-acf6b2393720", "8f279b07-26aa-46f6-8c9a-2a132f0b1177", "dacbebb6-72c7-4d4c-9009-aa95eba16dd3", "135e77a7-79ef-45e8-b138-67e283a14cdd", "9972b7a8-df8a-4626-b298-1dd0d19e451e"]
    }
    actor "Ground Operator" {
        id: "d475ed7d-4644-4461-97d4-b52772c9b425"
        description: "The Ground Operator is an actor interacting with the IFE System. According to the [MCB] All Missions and Capabilities, its main mission is to Provide Administration Means and Runtime Management and he has the following capabilities : Update Entertainment Offer and Provide Access Management Control. He is allocated to the following system Functions ; Configure the System Update Media Content Launch and Analyze Tests Set Passenger Service Authorization There are 3 diagrams that allow a better understanding its interactions with the IFE System. It explains the functional exchanges between the Ground Operator and the IFE System at two levels of accuracy : [SAB] Top Level System Overview [SAB] Top Level System Overview [SAB] High Level System Overview with some FCs For further information, it is possible to consult other diagrams related to the Ground Operator such as [SFBD] All Actor Functions which shows his functions alongside those of the other actors or [SFBD] All System Functions."
        allocated_functions: ["88faafec-6848-4494-8dd3-2acda3e53688", "4d0046b4-4865-438d-b09b-fd95671db09b", "ff70c856-f667-4e61-98d7-00190703d53a", "5e0fff47-a1a5-4a13-867c-18348bd894af"]
    }
    functional_chain "Watch Imposed Video on Cabin Screen" {
        id: "dd56a4c7-8304-4471-bf3f-9ffddaccf544"
        owner: "291fb362-9ae5-4779-8405-1281ea3709f6"
        involves: ["d683afa5-907e-494d-8f35-e10db839feaf", "e1808524-dfa9-4883-a809-c44a87712d1b", "1bc70a69-c6ad-4203-abb9-88b644ac0b95", "6b9e59a9-89f2-497e-a348-3aef80689e61", "cfde8bb1-4df9-4573-a516-41df984ac753", "cfde8bb1-4df9-4573-a516-41df984ac753", "06b8dd9b-2199-4ad1-a9b2-229d1de1bb48"]
    }
    functional_chain "Watch Imposed Video on Private Screen" {
        id: "adb3033a-d058-4995-98e9-60aaf85f4328"
        owner: "291fb362-9ae5-4779-8405-1281ea3709f6"
        involves: ["cfde8bb1-4df9-4573-a516-41df984ac753", "06b8dd9b-2199-4ad1-a9b2-229d1de1bb48", "e1808524-dfa9-4883-a809-c44a87712d1b", "d8b8c1bd-a2ea-47fd-a7a9-0239408db1a5", "e1808524-dfa9-4883-a809-c44a87712d1b", "cfde8bb1-4df9-4573-a516-41df984ac753", "880520c2-6bd9-4481-b9da-6ccee7b9d665", "d683afa5-907e-494d-8f35-e10db839feaf"]
    }
    functional_chain "Stop Playing Imposed Video [NOT DONE]" {
        id: "7d375159-5cdb-4959-8e5c-98f43941240c"
        owner: "291fb362-9ae5-4779-8405-1281ea3709f6"
    }
    functional_chain "Pause Running Imposed Video [NOT DONE]" {
        id: "2c6157aa-a353-4db7-9f05-05bb1db08215"
        owner: "291fb362-9ae5-4779-8405-1281ea3709f6"
    }
    functional_chain "Broadcast Audio Announcement" {
        id: "f5c2a034-4749-41c2-a77e-6ecaae28f5d0"
        owner: "291fb362-9ae5-4779-8405-1281ea3709f6"
        involves: ["3398fdbf-3415-4d03-a457-477324d2e091", "8f279b07-26aa-46f6-8c9a-2a132f0b1177", "e1808524-dfa9-4883-a809-c44a87712d1b", "d8b8c1bd-a2ea-47fd-a7a9-0239408db1a5", "e04025f9-6b52-4377-bac1-78d0bf2f8fed", "7de3b04b-6291-4098-afbd-2fb72cd25022"]
    }
    functional_chain "Start Playing VOD Movie" {
        id: "e00b45ce-330b-4b19-ac67-07095a77f8c0"
        owner: "839ba873-8442-4a01-bd1a-1f4c3cc6ed52"
        involves: ["d8b8c1bd-a2ea-47fd-a7a9-0239408db1a5", "e1808524-dfa9-4883-a809-c44a87712d1b", "880520c2-6bd9-4481-b9da-6ccee7b9d665", "4dbd5747-9662-4378-88cf-270b69ef3827", "5e221dee-8dc7-4a29-9224-ab53fedb3995", "d683afa5-907e-494d-8f35-e10db839feaf"]
    }
    functional_chain "Resume VOD Movie" {
        id: "5ecacd76-5aa1-4a9f-91f5-e00bad983206"
        owner: "839ba873-8442-4a01-bd1a-1f4c3cc6ed52"
        involves: ["4dbd5747-9662-4378-88cf-270b69ef3827", "5e221dee-8dc7-4a29-9224-ab53fedb3995", "e1808524-dfa9-4883-a809-c44a87712d1b", "880520c2-6bd9-4481-b9da-6ccee7b9d665", "d8b8c1bd-a2ea-47fd-a7a9-0239408db1a5", "d683afa5-907e-494d-8f35-e10db839feaf"]
    }
    functional_chain "VOD Service Activation [NOT DONE]" {
        id: "0293672b-2202-4438-8ad4-86af57df02c3"
        owner: "839ba873-8442-4a01-bd1a-1f4c3cc6ed52"
    }
    functional_chain "VOD Movie Interruption [NOT DONE]" {
        id: "b69dc23e-97e5-4f8f-b4b4-704fcd851d9e"
        owner: "839ba873-8442-4a01-bd1a-1f4c3cc6ed52"
    }
    functional_chain "Stop VOD Movie[NOT DONE]" {
        id: "10a21953-49e6-4179-8ce8-aafc125e9540"
        owner: "839ba873-8442-4a01-bd1a-1f4c3cc6ed52"
    }
    mission "Provide Entertainement Solutions" {
        id: "67e5b193-060d-43ff-b9bb-7bd14bcf425a"
        involves: ["c8b78c78-5b11-4fc0-87b7-3ca84622efea", "181a678c-dca9-46c1-9d18-b5a0c457c0de", "e0a46739-7e50-4ea6-b198-0e5efb9ce68f"]
    }
    mission "Provide Cabin Management Solutions" {
        id: "eee7180d-9cb1-4253-9ecb-2e5c811db2c9"
        involves: ["4890eae8-7a62-4d7b-bf8a-1590a199eebf", "181a678c-dca9-46c1-9d18-b5a0c457c0de", "c8b78c78-5b11-4fc0-87b7-3ca84622efea", "e0a46739-7e50-4ea6-b198-0e5efb9ce68f"]
    }
    mission "Provide Test and Maintenance Means" {
        id: "3d3f045d-4c6e-4b2c-8550-5e1ddd6ce87a"
        involves: ["d475ed7d-4644-4461-97d4-b52772c9b425", "4890eae8-7a62-4d7b-bf8a-1590a199eebf", "c8b78c78-5b11-4fc0-87b7-3ca84622efea", "e0a46739-7e50-4ea6-b198-0e5efb9ce68f"]
    }
    mission "Provide Connectivity Solutions" {
        id: "e9691a85-30a6-4b72-842d-b0e59ca2a104"
        description: "[Wikipedia] In recent years, IFE has been expanded to include in-flight connectivity—services such as Internet browsing, text messaging, cell phone usage (where permitted), and emailing. In fact, some in the airline industry have begun referring to the entire in-flight-entertainment category as \"IFEC\" (In-Flight Entertainment and Connectivity or In-Flight Entertainment and Communication)."
        involves: ["181a678c-dca9-46c1-9d18-b5a0c457c0de", "c8b78c78-5b11-4fc0-87b7-3ca84622efea", "e0a46739-7e50-4ea6-b198-0e5efb9ce68f"]
    }
    mission "Provide Administration Means and Runtime Management" {
        id: "cdfe2a5a-5069-4055-be2c-96be11552fa9"
        involves: ["e0a46739-7e50-4ea6-b198-0e5efb9ce68f"]
    }
    capability "Provide Audio and Video Intercommunication Means" {
        id: "291fb362-9ae5-4779-8405-1281ea3709f6"
        mission: "eee7180d-9cb1-4253-9ecb-2e5c811db2c9"
        involves: ["4890eae8-7a62-4d7b-bf8a-1590a199eebf", "181a678c-dca9-46c1-9d18-b5a0c457c0de", "c8b78c78-5b11-4fc0-87b7-3ca84622efea", "e0a46739-7e50-4ea6-b198-0e5efb9ce68f", "5e0fff47-a1a5-4a13-867c-18348bd894af", "5e221dee-8dc7-4a29-9224-ab53fedb3995", "d8b8c1bd-a2ea-47fd-a7a9-0239408db1a5", "880520c2-6bd9-4481-b9da-6ccee7b9d665", "d683afa5-907e-494d-8f35-e10db839feaf", "e1808524-dfa9-4883-a809-c44a87712d1b", "4dbd5747-9662-4378-88cf-270b69ef3827", "cfde8bb1-4df9-4573-a516-41df984ac753", "6e577fe5-ebd7-409e-b5de-b876fd689851", "e04025f9-6b52-4377-bac1-78d0bf2f8fed", "6b9e59a9-89f2-497e-a348-3aef80689e61", "8f279b07-26aa-46f6-8c9a-2a132f0b1177", "2c6157aa-a353-4db7-9f05-05bb1db08215", "7d375159-5cdb-4959-8e5c-98f43941240c", "dd56a4c7-8304-4471-bf3f-9ffddaccf544", "f5c2a034-4749-41c2-a77e-6ecaae28f5d0", "adb3033a-d058-4995-98e9-60aaf85f4328"]
    }
    capability "Provide Video Entertainment Services" {
        id: "839ba873-8442-4a01-bd1a-1f4c3cc6ed52"
        description: "[Wikipedia] Video entertainment is provided via a large video screen at the front of a cabin section, as well as via smaller monitors situated every few rows above the aisles. Sound is supplied via the same headphones as those distributed for audio entertainment. Personal televisions (PTVs) for every passenger provide passengers with channels broadcasting new and classic films, as well as comedies, news, sports programming, documentaries, children's shows, and drama series. Some airlines also present news and current affairs programming, which are often pre-recorded and delivered in the early morning before flights commence. PTVs are operated via an In flight Management System which stores pre-recorded channels on a central server and streams them to PTV equipped seats during flight. VOD (Video On Demand) systems store individual programs separately, allowing a passenger to have a specific program streamed to them privately, and be able to control the playback."
        mission: "67e5b193-060d-43ff-b9bb-7bd14bcf425a"
        involves: ["181a678c-dca9-46c1-9d18-b5a0c457c0de", "e0a46739-7e50-4ea6-b198-0e5efb9ce68f", "65787ee3-ab55-4200-91e3-d772a4ee3f28", "d8b8c1bd-a2ea-47fd-a7a9-0239408db1a5", "880520c2-6bd9-4481-b9da-6ccee7b9d665", "e1808524-dfa9-4883-a809-c44a87712d1b", "8a4b7443-4f73-49c2-a175-940a5d8accea", "d683afa5-907e-494d-8f35-e10db839feaf", "4dbd5747-9662-4378-88cf-270b69ef3827", "1bc70a69-c6ad-4203-abb9-88b644ac0b95", "5e0fff47-a1a5-4a13-867c-18348bd894af", "5e221dee-8dc7-4a29-9224-ab53fedb3995", "b69dc23e-97e5-4f8f-b4b4-704fcd851d9e", "0293672b-2202-4438-8ad4-86af57df02c3", "10a21953-49e6-4179-8ce8-aafc125e9540", "5ecacd76-5aa1-4a9f-91f5-e00bad983206", "e00b45ce-330b-4b19-ac67-07095a77f8c0"]
        realizes: "2569c843-e368-451c-b7be-67af191a35a3"
    }
    capability "Provide Video Gaming Services" {
        id: "b3a96943-f9e1-4249-97ce-c974ed889f41"
        description: "[Wikipedia] Video games are another emerging facet of in-flight entertainment. Some game systems are networked to allow interactive playing by multiple passengers. Later generations of IFE games began to shift focus from pure entertainment to learning. The best examples of this changing trend are the popular trivia game series and the Berlitz Word Traveler that allows passengers to learn a new language in their own language. Appearing as a mixture of lessons and mini games, passengers can learn the basics of a new language while being entertained"
        mission: "67e5b193-060d-43ff-b9bb-7bd14bcf425a"
        involves: ["181a678c-dca9-46c1-9d18-b5a0c457c0de", "e0a46739-7e50-4ea6-b198-0e5efb9ce68f"]
        realizes: "2569c843-e368-451c-b7be-67af191a35a3"
    }
    capability "Provide Wi-Fi Connectivity Service" {
        id: "67bacc43-8066-4b96-8101-cf8f3c5f9b35"
        description: "[Wikipedia] In-flight internet service is provided either through a satellite network or an air-to-ground network. Data communication via satellite system allows passengers to connect to live internet from the individual IFE units or their laptops via the in-flight Wi-Fi access"
        mission: "e9691a85-30a6-4b72-842d-b0e59ca2a104"
        involves: ["181a678c-dca9-46c1-9d18-b5a0c457c0de", "e0a46739-7e50-4ea6-b198-0e5efb9ce68f"]
    }
    capability "Provide Satellite and Internal Telephony Services" {
        id: "43207098-1d88-4680-ae14-68f165e74c24"
        description: "[Wikipedia] Some airlines provide satellite telephones integrated into their system. These are either found at strategic locations in the aircraft or integrated into the passenger remote control used for the individual in-flight entertainment. Passengers can use their credit card to make phone calls anywhere on the ground. These systems are usually not capable of receiving incoming calls. There are also some aircraft that allow faxes to be sent and the rate is usually the same as the call rate, but at a per page rate. Some systems also allow the transmission of SMS. More modern systems allow passengers to call fellow passengers located in another seat by simply keying in the recipient's seat number."
        mission: "e9691a85-30a6-4b72-842d-b0e59ca2a104"
        involves: ["181a678c-dca9-46c1-9d18-b5a0c457c0de", "c8b78c78-5b11-4fc0-87b7-3ca84622efea", "e0a46739-7e50-4ea6-b198-0e5efb9ce68f"]
    }
    capability "Provide Moving-Map Services" {
        id: "4e6fa11e-933f-4aec-b4c0-621c8fb244e5"
        description: "[Wikipedia] A moving-map system is a real-time flight information video channel broadcast through to cabin project/video screens and personal televisions (PTVs). In addition to displaying a map that illustrates the position and direction of the plane, the system gives the altitude, airspeed, outside air temperature, distance to the destination, distance from the origination point, and local time. The moving-map system information is derived in real time from the aircraft's flight computer systems."
        mission: "67e5b193-060d-43ff-b9bb-7bd14bcf425a"
        involves: ["c8b78c78-5b11-4fc0-87b7-3ca84622efea", "181a678c-dca9-46c1-9d18-b5a0c457c0de", "e0a46739-7e50-4ea6-b198-0e5efb9ce68f"]
    }
    capability "Provide Audio Entertainment Services" {
        id: "4cfe1222-e1e6-4af5-bcb7-b6a51b4ae50c"
        description: "[Wikipedia] Audio entertainment covers music, as well as news, information, and comedy. Most music channels are pre-recorded and feature their own DJs to provide chatter, song introductions, and interviews with artists. In addition, there is sometimes a channel devoted to the plane's radio communications, allowing passengers to listen in on the pilot's in-flight conversations with other planes and ground stations."
        mission: "67e5b193-060d-43ff-b9bb-7bd14bcf425a"
        involves: ["181a678c-dca9-46c1-9d18-b5a0c457c0de", "e0a46739-7e50-4ea6-b198-0e5efb9ce68f"]
        realizes: "2569c843-e368-451c-b7be-67af191a35a3"
    }
    capability "Provide Personal Device Connectivity" {
        id: "9a942752-caaa-4a14-9e18-fca348022a41"
        mission: "e9691a85-30a6-4b72-842d-b0e59ca2a104"
        involves: ["181a678c-dca9-46c1-9d18-b5a0c457c0de", "e0a46739-7e50-4ea6-b198-0e5efb9ce68f"]
    }
    capability "Provide Testing Interface" {
        id: "1adec5b0-a145-4a8f-b6c3-35f0fe1ae2d5"
        mission: "3d3f045d-4c6e-4b2c-8550-5e1ddd6ce87a"
        involves: ["c8b78c78-5b11-4fc0-87b7-3ca84622efea", "d475ed7d-4644-4461-97d4-b52772c9b425", "e0a46739-7e50-4ea6-b198-0e5efb9ce68f"]
    }
    capability "Provide Configuration Means" {
        id: "a643f8af-0104-49c8-add7-525806988ccb"
        mission: "3d3f045d-4c6e-4b2c-8550-5e1ddd6ce87a"
        involves: ["d475ed7d-4644-4461-97d4-b52772c9b425", "4890eae8-7a62-4d7b-bf8a-1590a199eebf", "e0a46739-7e50-4ea6-b198-0e5efb9ce68f"]
    }
    capability "Update Entertainment Offer" {
        id: "32ff49c3-04bc-476e-b03e-bb6ecbcfe554"
        mission: "cdfe2a5a-5069-4055-be2c-96be11552fa9"
        involves: ["d475ed7d-4644-4461-97d4-b52772c9b425", "e0a46739-7e50-4ea6-b198-0e5efb9ce68f"]
    }
    capability "Provide Access Management Control" {
        id: "74e6e35e-b1e7-45c1-bd9f-5d80c0714472"
        mission: "cdfe2a5a-5069-4055-be2c-96be11552fa9"
        involves: ["d475ed7d-4644-4461-97d4-b52772c9b425", "181a678c-dca9-46c1-9d18-b5a0c457c0de", "e0a46739-7e50-4ea6-b198-0e5efb9ce68f", "3f8567f2-5389-42c0-84a9-8a1360dc9572", "2c453928-a098-4f8a-a1e6-fc545a2840d1", "b7244d6a-08ab-4455-bbd9-aa5f1bbc9b5b", "65787ee3-ab55-4200-91e3-d772a4ee3f28", "0a2099f5-2831-4beb-89ae-996a6f54d032"]
    }
    capability "Integrate Aircraft Constraints" {
        id: "5acd248a-3961-4e71-85a4-5bc3b775215f"
        involves: ["c8b78c78-5b11-4fc0-87b7-3ca84622efea", "e0a46739-7e50-4ea6-b198-0e5efb9ce68f", "9972b7a8-df8a-4626-b298-1dd0d19e451e", "3f8567f2-5389-42c0-84a9-8a1360dc9572", "b7244d6a-08ab-4455-bbd9-aa5f1bbc9b5b", "135e77a7-79ef-45e8-b138-67e283a14cdd"]
    }
}

logical_architecture "Logical Architecture" {
    id: "cab89820-127c-4d24-bfe7-31b2000cc23c"
    interface "Streaming Protocol" {
        id: "5068b468-3030-4071-b6c3-ee4c55bc912b"
        level: "physical"
        exchange_items: ["b1ca4bf5-1b54-4f5c-a385-9514227695d3", "b65b0dee-34a9-4397-9626-e8095e348482"]
    }
    interface "Media Request" {
        id: "8336c5d9-2c65-48ec-b56b-d9432cd79a63"
        level: "physical"
        exchange_items: ["32703aff-a875-41e8-a93c-20e8a4b8df4b"]
    }
    interface "Media Files" {
        id: "ccf5b594-6b38-4c7f-8c92-5f592f9181ab"
        level: "physical"
        exchange_items: ["a34141d6-27d6-4e31-9994-828ee1560ded"]
    }
    interface "VOD Movie Control Command" {
        id: "dd7fd975-8352-46f4-97c1-b61b23c9c832"
        level: "physical"
        exchange_items: ["d3b096f5-9eb4-4c4a-b0af-946a71992c54"]
    }
    component "IFE System" {
        id: "724d6db9-9175-450b-8f05-023fcffec79c"
        component "Aircraft Front Servers" {
            id: "_8i1cwI2bEd6Q36a6kYkaHQ"
            component "Aircraft Interface" {
                id: "_gzH_AI2cEd6Q36a6kYkaHQ"
                port inout "CP 3" { id: "10e3cfb4-54a0-491c-9f4b-0f626f17393b" }
                port in "CP 4" { id: "e198c90c-b5be-4dc5-a5d0-509c39418ffe" }
                component "Aircraft-Specific Aircraft Interface" {
                    id: "_SF1a8JY-Ed6rH5SGn-UjSg"
                    function "Acquire Audio Stream from Aircraft" {
                        id: "ee920873-7e2d-487f-a51e-76236d6b094e"
                        description: "Audio announcements are performed through the aricraft built-in sound system (microphone, speakers, etc.). The IFE system is responsible for dispatching these announcements to each Seat TV. This function receives the audio streams from the aircraft and forwards it to the IFE applicative server. The implementation of this function is likely to be specific to each aircraft."
                        realizes: "e1808524-dfa9-4883-a809-c44a87712d1b"
                        parent: "c268e3f1-3cbc-4e62-8ae2-75975bc4481f"
                        port in "FIP 1" { id: "7961b385-49cd-4078-94f7-5ed8d27bd55e" }
                        port out "FOP 1" { id: "2be0f8fc-b7ee-47f5-80b1-e4a3dba102d3" }
                    }
                    function "Capture Aircraft Notifications and Parameters" {
                        id: "4b01ac5e-7920-491d-af35-1846163e5ad9"
                        description: "This function is concentrating the reception of the messages / notifications sent by the aircraft. The implementation of this function is likely to be specific to each aircraft model: The format of the incoming messages is defined by the aircraft manufacturer."
                        realizes: "3f8567f2-5389-42c0-84a9-8a1360dc9572"
                        parent: "c268e3f1-3cbc-4e62-8ae2-75975bc4481f"
                        port in "FIP 1" { id: "f4eeb86b-cab4-4b4f-b4bc-b1481aa3d029" }
                        port in "FIP 2" { id: "b400e0fd-1692-4749-9489-823b35cf598a" }
                        port out "FOP 1" { id: "4d9f4729-0475-409c-ab2f-3622d5dcbe62" }
                        port out "FOP 2" { id: "8c15fdb0-7ac9-435e-9cf3-5d998d95c5c7" }
                    }
                    function "Forward Audio Stream to Aircraft Cabin Speakers" {
                        id: "30d03eaa-2615-481c-bf1f-53f177d8147c"
                        description: "This function receives the audio track of imposed videos and forward them to the aircraft. The video part is played on the Cabin Screen through another dedicated function."
                        realizes: "e1808524-dfa9-4883-a809-c44a87712d1b"
                        parent: "c268e3f1-3cbc-4e62-8ae2-75975bc4481f"
                        port in "FIP 1" { id: "34d2a18b-7914-4179-9f28-eaa8d336c2d4" }
                        port out "FOP 1" { id: "11392ffc-fb07-4dbf-b2fb-901cfd933a4d" }
                    }
                }
                component "Core Aircraft Interface" {
                    id: "_Rml6MJY-Ed6rH5SGn-UjSg"
                    function "Determine Operating Profiles" {
                        id: "1e812d3d-6c35-42c9-80fa-eb64dfc1a3a8"
                        description: "Based on the notifications it receives from the function interfacing with the aircraft, this function computes the mode in which the IFE system has to operate. For example, if the aircraft sends a power reduction request, the IFE may switch to a downgraded mode."
                        realizes: "3f8567f2-5389-42c0-84a9-8a1360dc9572"
                        parent: "c268e3f1-3cbc-4e62-8ae2-75975bc4481f"
                        port in "FIP 1" { id: "c766520c-30f8-450b-a0de-f008b1b26ea7" }
                        port in "FIP 2" { id: "181c5e80-8e04-441f-823a-e2d8f04d3a3b" }
                        port out "FOP 1" { id: "1d0ff53e-3f71-4c14-a286-b63d22d7df14" }
                    }
                }
            }
            component "Media Server" {
                id: "_JNwHUJY7Ed6rH5SGn-UjSg"
                port out "CP 1" { id: "f547aac3-ff7d-4aed-ad0d-5d7c2c991a45" }
                port inout "CP 2" { id: "4c2be455-b669-4079-8fe4-0ad0ea898db6" }
                function "Load Digital Media" {
                    id: "70d62f70-d0b6-491c-9e9e-f5ed77179ce4"
                    description: "This function is responsible for receiving the media content uploaded by the ground operator and for moving it onto the IFE media storage unit."
                    realizes: "a16d1d72-9db2-4554-af57-411b29793904"
                    parent: "2a6d57b4-97f3-4344-b645-cefcc9462708"
                    port in "FIP 1" { id: "a7b597c9-ff1d-4d4a-94df-97177da442af" }
                    port out "FOP 1" { id: "9ad04ab4-c015-4d2f-92ff-3fdd2d4e0722" }
                }
                function "Store Digital Media" {
                    id: "be9b5663-23ea-4413-abeb-3d5df84c93b8"
                    description: "This function is responsible for the storage of audio and video files that will be streamed and of movie information such as metadata (pictures,description)."
                    status: "REWORK NECESSARY"
                    review: "Supprimer les sous fonctions?"
                    realizes: "d683afa5-907e-494d-8f35-e10db839feaf"
                    parent: "2a6d57b4-97f3-4344-b645-cefcc9462708"
                    port in "FIP 1" { id: "869c781f-a34e-467d-9c98-46f50f205184" }
                    port in "FIP 2" { id: "b14afffd-bd85-45a2-9171-a1bfe16c5156" }
                    port in "FIP 3" { id: "f60cd39f-7a54-4dd6-a6e6-ee6371590571" }
                    port in "FIP 4" { id: "02ede5ec-1d97-4072-bf78-206a5499f402" }
                    port out "FOP 1" { id: "72849d55-1089-4e07-9e6e-4b52dd81c2c5" }
                    port out "FOP 2" { id: "95898321-035b-463c-a16d-5504346c9210" }
                    port out "FOP 3" { id: "ca1f9e4b-721e-403c-bac2-4459af1b31a9" }
                    port out "FOP 4" { id: "240cda30-7e66-482d-99db-a6a9da88ca28" }
                    port out "FOP 5" { id: "f2501b6b-2582-400b-82f0-cbd3eb49ea7b" }
                }
            }
            component "Streaming Server" {
                id: "_QHUAIJY7Ed6rH5SGn-UjSg"
                port out "CP 1" { id: "54c5ae2d-bf5a-41a7-96ba-93fb449184af" }
                port out "CP 2" { id: "455649cf-4913-4972-bec7-14a0ac9f8d6d" }
                port inout "CP 3" { id: "974324e8-4090-4c22-a8e0-46eb1fb39424" }
                port in "CP 4" { id: "c781fc0c-fbb5-489e-83dc-1def672b03b4" }
                function "Broadcast Audio Video Stream" {
                    id: "61bf6852-3f28-425d-b8ed-4d38b2b4eebb"
                    description: "This function consists in transmitting audio and video streams to one or several Seat TVs and or to the cabin screens. The implementation of this function is likely to be based on streaming solutions."
                    realizes: "e1808524-dfa9-4883-a809-c44a87712d1b"
                    parent: "d4ee89c8-dcf8-4fa5-b276-0afa03d409c9"
                    port in "FIP 2" { id: "33b0bdc6-ea7c-490c-9ae4-d80f9d792686" }
                    port out "FOP 1" { id: "f027a07d-c94a-40ad-bf92-bd9bb8b81777" }
                    port out "FOP 2" { id: "fad9d362-fc60-4515-8d1c-65ba84265d92" }
                    port out "FOP 3" { id: "63825566-e6a7-44cc-ab37-7034996e11e3" }
                }
                function "Prepare Broadcasts" {
                    id: "27d03649-545b-40b0-8407-721165539005"
                    description: "Thiis function is responsible for preparing broadcast material: Retrieving the digital content to be broadcasted from the storage unit and transmitting information about the target(s) of the broadcast."
                    realizes: "e1808524-dfa9-4883-a809-c44a87712d1b"
                    parent: "d4ee89c8-dcf8-4fa5-b276-0afa03d409c9"
                    port in "FIP 1" { id: "e6686dbe-8882-45e4-954e-8eaaf0ba280b" }
                    port in "FIP 3" { id: "a2af641b-db0e-4d01-90aa-d3e1b5decd6f" }
                    port in "FIP 2" { id: "0f3bef74-c269-479b-95c1-416f21cf31b9" }
                    port out "FOP 1" { id: "1ef429c2-560d-414c-bee7-d3339a902274" }
                    port out "FOP 3" { id: "1932ce43-8838-46f3-aa5a-9523f628819f" }
                    port out "FOP 4" { id: "5e9eaf56-6419-4fb4-8d96-e5ffc6559a75" }
                    port out "FOP 2" { id: "fbc6523a-dda1-4086-b3bf-0e77bba2454b" }
                }
            }
            component "Applications Server" {
                id: "df043998-7fd1-4d4d-899f-7257788d644f"
                port inout "CP 1" { id: "d3a9f7dc-14e8-45e7-8f4c-b7155f6dcc04" }
                port inout "CP 2" { id: "9ae956ba-2745-457c-ad69-25158ab8e09b" }
                port out "CP 3" { id: "f1dfce90-60e3-4f2d-8327-5b9d9d8b78ac" }
                function "Determine  Passenger Service Availability" {
                    id: "5a58fd72-1b57-41dc-8b7b-b0b8cf577236"
                    description: "This function is responsible for computing the availability of passenger services based on the authorization scheme (airline business strategy) and the aircraft contraints."
                    available_in_states: ["6d479123-ab40-4f72-a21d-b28be1c0ad54", "39eb29ec-2085-47fe-b3ea-55027fc82faf"]
                    realizes: "b7244d6a-08ab-4455-bbd9-aa5f1bbc9b5b"
                    parent: "11f3e0e8-a78d-4962-bed8-9672b1167331"
                    port in "FIP 1" { id: "288ffb0d-824c-4a3e-81a6-6c876d3fd2d1" }
                    port in "FIP 2" { id: "1572986c-3e6d-4878-b8a0-2bc0ec91895b" }
                    port in "FIP 3" { id: "54d0a401-1d68-41a1-9f85-1a69c15ac5bc" }
                    port out "FOP 1" { id: "feb33041-b2ae-49a9-ba21-0129bc63127d" }
                }
                function "Process Audio Announcement" {
                    id: "39f96e89-8ead-4742-bc1f-6eb0dd33990b"
                    description: "When an audio announcement is made (thoufh the aircraft equipment), the IFE system is responsible for forwarding the audio stream to all passengers. This function also triggers a passenger service interruption command"
                    realizes: "e1808524-dfa9-4883-a809-c44a87712d1b"
                    parent: "95495bbc-987c-4b17-95d0-9aa5a87f0344"
                    port in "FIP 1" { id: "d604d5bd-d8a5-42eb-9091-9c4e26243634" }
                    port out "FOP 3" { id: "87bf2b23-f3ba-479b-b180-70a80e8ec7f0" }
                    port out "FOP 4" { id: "03979293-d07a-478c-968c-0fe7bbcd966a" }
                }
            }
        }
        component "Seat TV" {
            id: "_8rBvcI2bEd6Q36a6kYkaHQ"
            component "Seat TV Interactions Manager" {
                id: "_neToII2dEd6Q36a6kYkaHQ"
                port in "CP 3" { id: "6328d4b6-d111-4c3c-b162-3709a7d79250" }
                port inout "CP 4" { id: "83c4ff0c-a669-470b-b159-d537762f7456" }
                port out "CP 5" { id: "a61aa702-5251-4a4b-9661-acea2aa9becc" }
                component "Seat TV Core Interactions Manager" {
                    id: "_2SUXEJZFEd6rH5SGn-UjSg"
                    function "Handle VOD Service Activation and Interruption" {
                        id: "910a61f1-3da0-404f-9db5-c226af9a3c47"
                        description: "This function is responsible for managing the VOD Service activations / deactivation (first activation, service resume, external interruption, etc.). This function relies on another function in charge of storing the current status of the interrupted / deactivated service (in particular, the current selection movie and the current position in this movie)."
                        realizes: "d8b8c1bd-a2ea-47fd-a7a9-0239408db1a5"
                        parent: "7a8c51b2-c42f-49cd-a003-752d0de0b4da"
                        port in "FIP 1" { id: "7b084f05-ff4e-4875-89dd-7d4acd4a7e23" }
                        port in "FIP 2" { id: "9855a1d4-52de-4ed5-83d5-4aac50f911d1" }
                        port in "FIP 3" { id: "defdafc3-a332-459b-9fb4-2d74b96abaaa" }
                        port out "FOP 1" { id: "05679dee-1c1b-4ce8-b71d-924b76f2cd43" }
                        port out "FOP 2" { id: "bb3bc214-eb95-4b9b-936c-9679d84ac1d6" }
                        port out "FOP 3" { id: "9c8a1788-451c-4e3a-bbe1-34810ab3358e" }
                    }
                    function "Process VOD Movie Controls" {
                        id: "faf3223f-3f47-4ad6-9c58-353b34e1a30f"
                        description: "This function transforms the VOD Movie play/pause/stop selections into applicative commands. The information about which Seat TV is requesting is attached to the created command."
                        realizes: "5e221dee-8dc7-4a29-9224-ab53fedb3995"
                        parent: "a8452b58-3135-457b-8d85-59bd23240297"
                        port in "FIP 1" { id: "ec3eefa6-91dd-4251-9f3c-b41820c59b14" }
                        port in "FIP 2" { id: "95bb8c38-d66f-4a9c-b183-45d97f4b1d14" }
                        port in "FIP 3" { id: "862f4165-14c9-4d04-906e-c9741036eb7b" }
                        port out "FOP 2" { id: "23257d6e-d32c-4154-a67c-899ed3d03496" }
                    }
                    function "Retrieve VOD Movie Data" {
                        id: "651179c3-78b6-4a6f-a174-74405b81b36a"
                        description: "This function consists in receiving media information requests (list of VOD movies, available languages, etc.) and returning the data corresponding to these requests. This data is retrieved from the digital media storage unit."
                        realizes: "0a2099f5-2831-4beb-89ae-996a6f54d032"
                        parent: "7a8c51b2-c42f-49cd-a003-752d0de0b4da"
                        port in "FIP 1" { id: "7a77dbdf-cb7b-47c1-973c-20e6dd04a76c" }
                        port in "FIP 2" { id: "29f47e7b-c5fb-4876-a851-209c5699e6a8" }
                        port in "FIP 3" { id: "d7638d34-5fc8-463f-9ddf-5639ed426763" }
                        port in "FIP 4" { id: "b068d566-02e1-482f-874a-94eb7d4a2590" }
                        port in "FIP 5" { id: "9d8b9685-beb6-456b-b5af-03df09a23454" }
                        port in "FIP 6" { id: "7ccdb518-cc72-4dc1-982e-6e8f6b5444d0" }
                        port in "FIP 7" { id: "493a03ee-d29a-40f1-b2f9-607f0f5e7f52" }
                        port in "FIP 8" { id: "0b08d6c4-1169-4e98-9850-59137f04a062" }
                        port out "FOP 1" { id: "7da76bba-c49f-485b-bbdf-0cec72263ee3" }
                        port out "FOP 2" { id: "cea0cf85-33e3-4488-ab07-cb373d1882b2" }
                        port out "FOP 3" { id: "a5c87fdf-b091-48ae-804a-59f4124cf8d9" }
                        port out "FOP 4" { id: "44859e44-bc4d-4e26-b60c-d2776a52e364" }
                        port out "FOP 5" { id: "88c29017-b590-47cc-94b0-fcb14dea71b9" }
                    }
                    function "Start/Resume Service" {
                        id: "c3b17f24-ae52-413f-ace7-6baab1287b02"
                        description: "This function displays the available services and trigger service activations when applicable."
                        realizes: "2c453928-a098-4f8a-a1e6-fc545a2840d1"
                        parent: "11f3e0e8-a78d-4962-bed8-9672b1167331"
                        port in "FIP1" { id: "80b41c8f-b59a-4b31-9a8c-2b0aa1162a86" }
                        port in "FIP 2" { id: "bbe3d60c-3027-4697-9a8c-c8c9e3adb775" }
                        port in "FIP 4" { id: "7ad5b93c-59f0-4491-b3b6-4dcd14f939ab" }
                        port out "FOP 1" { id: "292453d3-dee5-4699-8e42-32a9e3ca9cc8" }
                        port out "FOP 4" { id: "d0efd2e1-ab41-4e83-99d3-d698866ceffe" }
                    }
                }
                component "Seat TV Airline-Specific Interactions Manager" {
                    id: "281c331f-1184-4c7d-afb2-7a55ff09a80f"
                    function "Capture Passenger Service Selection" {
                        id: "2ea2703a-6dd1-4835-aaae-16179864ac1d"
                        description: "This function&#160;handles the general interactions with the passenger. The Seat TV displays&#160;interaction widgets (typically buttons&#160;and slide bars). This typically airline-specifc, because user interface typically is. This function is responsible&#160;for&#160;capturing the selections of&#160;services and forwarding them to&#160;a more generic function&#160;(a controler) that will transform the selections into an internal system commands (activation / de-activation).&#160;&#160;"
                        realizes: "2c453928-a098-4f8a-a1e6-fc545a2840d1"
                        parent: "11f3e0e8-a78d-4962-bed8-9672b1167331"
                        port in "FIP 1" { id: "a4a24937-7365-4018-8951-9aeb80e8d835" }
                        port in "FIP1" { id: "bd1b2809-5faf-43bf-99d5-5a643e6cf532" }
                        port out "FOP 1" { id: "ad5f2949-cbd1-4081-bd55-632c22bd7b99" }
                        port out "FOP 2" { id: "9329cea3-1798-4c24-a4cb-3bd6434f95f2" }
                        port out "FOP1" { id: "37cb6a83-d4f5-4062-b7f2-6a021e26419d" }
                    }
                    function "Capture VOD Selections" {
                        id: "5165a225-e662-4b4a-95cd-c94fc5726524"
                        description: "This function&#160;handles the interactions with the passenger in the VOD service. The Seat TV displays&#160;interaction widgets (typically buttons&#160;and slide bars). This typically airline-specifc, because user interface typically is. This function is responsible&#160;for&#160;capturing these selections and forwarding them to&#160;a more generic function&#160;(a controler) that will transform the selection into an internal system command. &#160;"
                        realizes: "5e221dee-8dc7-4a29-9224-ab53fedb3995"
                        parent: "7a8c51b2-c42f-49cd-a003-752d0de0b4da"
                        port in "FIP 1" { id: "561c119c-6849-4b24-bb9d-4ddb87141c81" }
                        port in "FIP 2" { id: "28c1db11-b237-48ac-b482-c4187e0963d5" }
                        port in "FIP 3" { id: "44a464a9-a66c-48f5-b9d0-306aa27ab84f" }
                        port in "FIP 4" { id: "76b5100c-5027-4754-bf67-7812dc35b8ba" }
                        port in "FIP 5" { id: "cddcd8f3-4976-49e3-b291-9f755f8c0126" }
                        port out "FOP 1" { id: "abf8e721-bd5e-43bc-8c63-38dbdae8c212" }
                        port out "FOP 2" { id: "d30b9c62-fa66-49e6-87ba-55b985aab025" }
                        port out "FOP 3" { id: "6256b32b-1dbf-4e31-bc8e-f85e60fe16d2" }
                        port out "FOP 4" { id: "d4039031-ce6d-4712-ae52-a01d37fe7dbd" }
                        port out "FOP 5" { id: "d680a79a-04c1-4e71-be54-32496c0c0697" }
                        port out "FOP 6" { id: "96cf2cf1-b207-4fc7-932a-8c05c51fbca5" }
                        port out "FOP 7" { id: "60a91a7a-4d67-4ca5-b1e8-85ea7505733e" }
                    }
                    function "Display Audio Interruption Screen on Seat TV" {
                        id: "4ff700ef-4f6b-4151-917b-e59064e5a225"
                        description: "When an audio announcement is made, all IFE services on Seat TVs are interrupted. A message is then displayed on the screen, indicating that an audio annoucement is currently ongoing."
                        realizes: "d8b8c1bd-a2ea-47fd-a7a9-0239408db1a5"
                        parent: "cbe07842-a67d-488a-bf29-5b46d3369c15"
                        port in "FIP 1" { id: "9d60ec03-c1fd-4896-a3f7-8d40d57d0f6f" }
                        port out "FOP 1" { id: "2c3ab7c8-c3e7-4422-947a-40e7ed879df1" }
                    }
                    function "Display Homepage on Seat TV" {
                        id: "70f1acf3-a32d-4190-b97e-3eb361a559e7"
                        description: "This function simply displays the Seat TV starting page, from where all services can be launched."
                        realizes: "2c453928-a098-4f8a-a1e6-fc545a2840d1"
                        parent: "11f3e0e8-a78d-4962-bed8-9672b1167331"
                        port in "FIP 1" { id: "1c355682-fe46-4a3c-9900-07d093bf5585" }
                        port in "FIP 2" { id: "6f1bdfcf-1a52-4a91-a3ef-af7b8a70943c" }
                        port out "FOP1" { id: "d8fbff1d-471e-4f69-a865-dae9d719314a" }
                        port out "FOP 2" { id: "a1e12ff1-e0df-467a-9841-709d83be2d51" }
                    }
                    function "Display VOD Movie Data" {
                        id: "94aebe8b-af13-4a7a-a7ee-8fd10a7fe489"
                        description: "This function is responsible for displaying all the VOD service information on the Seat TV."
                        realizes: "5e221dee-8dc7-4a29-9224-ab53fedb3995"
                        parent: "7a8c51b2-c42f-49cd-a003-752d0de0b4da"
                        port in "FIP 2" { id: "a6c2f821-00ed-48f3-a038-c7b63f94916f" }
                        port in "FIP 1" { id: "3940479c-a41f-4087-83b5-31298de7f29b" }
                        port in "FIP 3" { id: "e68f8ee0-c024-45e1-8b2e-a1f10ba8305a" }
                        port in "FIP 4" { id: "fd6bdd1a-74e5-4f36-b771-a5af7757885a" }
                        port out "FOP 1" { id: "ca1f7ca3-706c-4dfb-a3d1-48e6bc674786" }
                        port out "FOP 2" { id: "4b72d40d-642d-4fe0-8d00-c31b2fe5357e" }
                        port out "FOP 4" { id: "4cf75134-953a-4974-aa78-64114562f296" }
                    }
                }
            }
            component "Seat TV Audio Video Player" {
                id: "_nxYVII2dEd6Q36a6kYkaHQ"
                port in "CP 1" { id: "4ac80683-f77f-43b3-8277-4cfd24802de2" }
                port in "CP 2" { id: "e4941019-68e5-431b-9816-58532003ae3e" }
                function "Play Audio-Video Stream on Seat TV" {
                    id: "7b604e03-85be-4d93-8035-8a84ef8086db"
                    description: "This function receives a media stream (either synchronized video and audio either audio only) and restitutes it through the Seat TV."
                    realizes: "e1808524-dfa9-4883-a809-c44a87712d1b"
                    parent: "cbe07842-a67d-488a-bf29-5b46d3369c15"
                    port in "FIP 1" { id: "08917ba1-7f54-4f20-941a-6c3985ab715e" }
                    port in "FIP 2" { id: "bf46fcb0-9d93-4e52-b762-1d1291e7bf0a" }
                    port in "FIP 3" { id: "8f35c196-0db0-4c52-b3da-3e07b89de0f6" }
                    port out "FOP 1" { id: "3eb92176-428c-4705-a971-e64bca911ff5" }
                    port out "FOP 3" { id: "957f3549-5aa2-48a0-93a3-df74cbe8c694" }
                }
                function "Store Interrupted VOD Movie Status" {
                    id: "3136218c-bf14-437d-99c9-6593352eead9"
                    description: "This function is in charge of storing the current status of the interrupted / deactivated VOD service (in particular, the current selection movie and the current position in this movie)."
                    realizes: "d8b8c1bd-a2ea-47fd-a7a9-0239408db1a5"
                    parent: "7a8c51b2-c42f-49cd-a003-752d0de0b4da"
                    port in "FIP 2" { id: "a7b46e44-7e42-44a0-b255-1e909158738e" }
                    port out "FOP 3" { id: "e6f08431-9dfa-4e3e-aec1-8b9fe47c016a" }
                }
            }
            component "Seat TV Services Manager" {
                id: "e02be1c5-a1a2-4c11-be30-259e92ef5826"
                port inout "CP 1" { id: "0c3f3763-3a98-43d2-b01f-e00ea8d473a5" }
                port in "CP 2" { id: "d294cbbe-55ee-4f00-bcd5-f605afd9fec4" }
                function "Interrupt Current Service" {
                    id: "b4623d17-6e62-411d-839c-2d768664a920"
                    description: "This function is responsible for triggering passenger service interruptions as soon as an imposed video is played or an audio announcement is performed. When a VOD movie is running, this means for example that the movie as to be automatically paused and the current position stored."
                    realizes: "d8b8c1bd-a2ea-47fd-a7a9-0239408db1a5"
                    parent: "11f3e0e8-a78d-4962-bed8-9672b1167331"
                    port in "FIP 3" { id: "fe5b214d-6777-4d19-8ad2-19c0fcd32768" }
                    port in "FIP 2" { id: "dddc9323-3db5-49ab-b33b-6cf9224d5a9f" }
                    port out "FOP1" { id: "e3e49f00-1931-4afb-9030-234c5407b5e0" }
                    port out "FOP 2" { id: "20fd8d77-8da5-47d8-b6af-a16137714a3d" }
                }
                function "Store Passenger Service State" {
                    id: "3e9ccc30-596e-40e6-8941-64838248dcee"
                    description: "Whenever a service is interrupted, its current state has to be saved, in order to be able to resume it later on in the same state. This function typically delegates this action to the functions directly responsible for running each passenger service."
                    realizes: "2c453928-a098-4f8a-a1e6-fc545a2840d1"
                    parent: "11f3e0e8-a78d-4962-bed8-9672b1167331"
                    port in "FIP 1" { id: "d3cc78af-43f0-4495-a35e-901e01ec0b8e" }
                    port in "FIP 2" { id: "0bd34514-a858-4945-9b96-116e3761009d" }
                    port out "FOP 3" { id: "9b4d3075-32fd-47a7-ba2d-ea51831f3d92" }
                }
            }
        }
        component "Cabin Screen" {
            id: "11e922f1-192b-43a5-9060-f935c26998a5"
            port in "CP 1" { id: "2570d77c-ed3a-4483-8e12-c43610f0c4d8" }
            function "Play Video Stream on Cabin Screen" {
                id: "2632b523-b577-48c5-afba-3eb4ccde3254"
                description: "This function receives a video stream and restitutes it through the Cabin screens. The audio track associated to the video is forwarded to the aircraft through another function."
                realizes: "e1808524-dfa9-4883-a809-c44a87712d1b"
                parent: "cbe07842-a67d-488a-bf29-5b46d3369c15"
                port in "FIP 1" { id: "cde0b963-94c4-4ac9-91f5-a86f9bacb6af" }
                port in "FIP 3" { id: "0a8c2bda-367b-446b-b2eb-cbf900570e36" }
                port out "FOP1" { id: "1248b6cf-fd44-41bc-8998-686fd51188c1" }
            }
        }
        component "Cabin Terminal" {
            id: "8dbc153d-c9e8-4cb5-93af-ecf9a0fc939c"
            component "Cabin Terminal Interactions Manager" {
                id: "_XnQJQJClEd6HJInKHsIvPg"
                port inout "CP 3" { id: "1ed2c19b-defa-4a37-9256-be5fb97eb6b1" }
                port inout "CP 4" { id: "dbe9027a-0da8-4ce1-9d2d-80308dadd634" }
                port out "CP 5" { id: "8aceb444-877d-4e2b-a8e1-63d60b9976ae" }
                component "Cabin Terminal Airline-Specific Interactions Manager" {
                    id: "97e2a56c-0d4c-4a56-924a-63a31e6c50d7"
                    function "Capture Imposed Video Selections" {
                        id: "1aa42ce5-1c2c-4473-9840-7d20a60c377e"
                        description: "This function&#160;handles the interactions with the&#160;cabin crew on the cabin management terminal.&#160;The terminal&#160;displays&#160;interaction widgets (typically buttons&#160;and slide bars). This typically airline-specifc, because user interface typically is. This function is responsible&#160;for&#160;capturing these selections and forwarding them to&#160;a more generic function&#160;(a controler) that will transform the selection into an internal system command."
                        realizes: "6e577fe5-ebd7-409e-b5de-b876fd689851"
                        parent: "81d017bc-85ca-4bcd-acc9-51423542c61e"
                        port in "FIP 1" { id: "8a0fcdfa-df41-4e4a-8603-4cc7d3de7242" }
                        port in "FIP 2" { id: "fc1343d0-1c4a-460e-baed-d628d4a0f5db" }
                        port in "FIP 3" { id: "2562a10c-7532-4db2-a012-9e7f3717c570" }
                        port in "FIP 4" { id: "72394ee8-41aa-4c15-bd12-d5b79404b713" }
                        port out "FOP 2" { id: "24041d21-e834-49d4-bc5c-39e7d4f10098" }
                        port out "FOP 3" { id: "17bdd689-0c10-4e37-9b3c-1cc7dbf27f32" }
                        port out "FOP 4" { id: "1d8a444a-af01-46f6-b387-6f5f054d4f2d" }
                        port out "FOP 5" { id: "483c1cc2-acf2-4947-85c7-430734bfeebe" }
                    }
                    function "Display Imposed Video Data" {
                        id: "a90e98e8-741e-478a-9e14-af7a50ee2525"
                        description: "This function is responsible for displaying all the imposed video data on the cabin management terminal."
                        realizes: "6e577fe5-ebd7-409e-b5de-b876fd689851"
                        parent: "81d017bc-85ca-4bcd-acc9-51423542c61e"
                        port in "FIP1" { id: "e8b56230-ab0b-4b28-b42a-ad20a2e95ae2" }
                        port out "FOP1" { id: "1f79b2c9-775c-48ba-a916-a23e656ed695" }
                        port out "FOP2" { id: "07879f4c-8a30-4a30-8841-f95026b08038" }
                    }
                    function "Display Imposed Video Playing Status on Cabin Terminal" {
                        id: "566f8900-7408-42d3-82f8-ba18bf0d923f"
                        description: "This function is responsible for displaying the status of the currently running imposed movie (ETA, etc.) on the Cabin Management Terminal."
                        realizes: "e1808524-dfa9-4883-a809-c44a87712d1b"
                        parent: "cbe07842-a67d-488a-bf29-5b46d3369c15"
                        port in "FIP 1" { id: "5ed638ed-ca65-4a18-b7b2-8d83717d74fb" }
                        port out "FOP 1" { id: "8b443b08-c0a5-4148-9d09-e72842ca1297" }
                    }
                }
                component "Cabin Terminal Core Interactions Manager" {
                    id: "01a67cde-a09c-446e-9637-ab235664d23c"
                    function "Process Imposed Video Controls" {
                        id: "7069fd31-6eb0-49fa-ae52-0b78307c2748"
                        description: "This function transforms the Imposed Video play/pause/stop selections into applicative commands."
                        realizes: "e1808524-dfa9-4883-a809-c44a87712d1b"
                        parent: "a8452b58-3135-457b-8d85-59bd23240297"
                        port in "FIP 2" { id: "ae96b3e7-7b7f-4e7b-9fd3-17c3063550f2" }
                        port in "FIP 4" { id: "9e1b1d99-8a13-48a4-82a2-126491d36629" }
                        port in "FIP 3" { id: "93681c76-0714-44ae-b4a9-f792992b735b" }
                        port out "FOP 3" { id: "221aeeca-a052-4f6b-92bd-9921ccbdb010" }
                        port out "FOP 4" { id: "6707ddc0-0c12-4545-a1d4-1dabb4989c8b" }
                    }
                    function "Receive Imposed Video Playing Status" {
                        id: "b55db2ae-a407-4c31-8093-75f0841094eb"
                        description: "This function collects the status of the currently running imposed movie (ETA, etc.). This status is then transmitted to be displayed on the Cabin Management Terminal."
                        realizes: "e1808524-dfa9-4883-a809-c44a87712d1b"
                        parent: "cbe07842-a67d-488a-bf29-5b46d3369c15"
                        port in "FIP 4" { id: "c1d0def2-6b2c-428d-839b-a9eb9633373b" }
                        port in "FIP 1" { id: "f65c50ac-1af7-4b34-ab1d-f83a6c8f173a" }
                        port out "FOP 1" { id: "d86ae730-d118-45a3-9c96-2571a9c9ef79" }
                    }
                    function "Retrieve Imposed Video Data" {
                        id: "0d727549-f01f-4cc6-ae27-5f25220ece06"
                        description: "This function consists in receiving media information requests (list of imposed videos, available languages, etc.) and returning the data corresponding to these requests. This data is retrieved from the digital media storage unit."
                        realizes: "6e577fe5-ebd7-409e-b5de-b876fd689851"
                        parent: "81d017bc-85ca-4bcd-acc9-51423542c61e"
                        port in "FIP 1" { id: "d273b057-8aae-47be-95a3-c349f89e306f" }
                        port in "FIP 2" { id: "44c2e3b2-4101-4b82-829d-65555dc1445c" }
                        port in "FIP 3" { id: "021e8ded-b63c-4372-a6f1-c58a3618e6ab" }
                        port out "FOP2" { id: "6bf3f1d3-567d-4a77-86a5-3973b1bac042" }
                        port out "FOP 2" { id: "cdec05b4-0557-4886-9e1a-15ad02cc7a32" }
                    }
                }
            }
        }
    }
    component "Passenger" {
        id: "395ee0ab-cb3d-48e8-a411-09a98053292f"
        actor: true
        function "Command VOD Session" {
            id: "bcb08500-a828-43b0-ac8b-ff4c7cb3b6b4"
            description: "This function is dedicated to the interactions in the context of the VOD service: Selection of movies (and languages) and control on these movies (play, pause, stop, etc.)."
            realizes: "4dbd5747-9662-4378-88cf-270b69ef3827"
            parent: "cf623512-1614-4548-9b4c-4c0e614c4516"
            port in "FIP 1" { id: "a6da5908-6f0a-49ae-8a3c-cac9e98ac061" }
            port in "FIP 2" { id: "13de6a3f-caf1-4911-b7b1-a6e79b7d9391" }
            port in "FIP 3" { id: "8f53e64f-7adb-43ea-840e-2f342e3d64c7" }
            port out "FOP 1" { id: "ffd82b3f-37fd-418a-899f-c1cdafc7c237" }
            port out "FOP 2" { id: "2edaee8c-1d76-4292-826b-6b3366fff7ff" }
            port out "FOP 3" { id: "24843551-9aa6-477f-84ef-f0440eacd0d3" }
            port out "FOP 4" { id: "9a9325f8-f8c5-43e1-90f8-5d9f16c24ab3" }
            port out "FOP 5" { id: "7301615b-612e-41c9-a2b1-bbaa6306ebe4" }
        }
        function "Listen to Audio Announcement" {
            id: "18da001e-dd8b-4041-ba4c-eb2211c624c4"
            description: "Audio announcements are information about the flight provided by the cabin crew or the captain (boarding, pre-flight announcements, captain's announcements, safety briefing, turbulence, take-off/ascent, descent/final, landing, etc.). The passenger can listen to these annoucements aither through the aircraft speakers either through the individual Seat TVs."
            realizes: "e04025f9-6b52-4377-bac1-78d0bf2f8fed"
            parent: "cf623512-1614-4548-9b4c-4c0e614c4516"
            port in "FIP 1" { id: "3254bd59-7213-4e7a-bd0d-dde4cbdeede9" }
            port in "FIP 2" { id: "79656cbc-9576-4f85-b3f7-60f2cfa8bbdd" }
            port in "FIP 3" { id: "668df250-eb8b-48b5-9e3f-d9cc1fe75974" }
        }
        function "Select Passenger Service" {
            id: "63136eb0-622b-43cc-b4da-55b86ae281c1"
            description: "This function covers the selection by the passenger of a given IFE service (gaming, moving-map, VOD, etc.)."
            realizes: "65787ee3-ab55-4200-91e3-d772a4ee3f28"
            parent: "cf623512-1614-4548-9b4c-4c0e614c4516"
            port in "FIP1" { id: "ecda097d-75b7-49d1-b8c1-478838e7b118" }
            port out "FOP1" { id: "a8e9d027-e37b-42b7-be68-27e315957e78" }
            port out "FOP 2" { id: "8a4862f6-3229-4eae-80db-0f324420cb6d" }
        }
        function "Watch Imposed Movie on Cabin Screen" {
            id: "295db719-853a-4cb6-9bbe-2e66b5ae9ace"
            description: "This function represents the fact that the passenger can watch a video on a cabin screen (typically located in the ceiling)."
            realizes: "1bc70a69-c6ad-4203-abb9-88b644ac0b95"
            parent: "cf623512-1614-4548-9b4c-4c0e614c4516"
            port in "FIP3" { id: "b9efef88-cc1e-4671-9429-4795ac4c9222" }
            port in "FIP2" { id: "23fb7577-0354-449f-9f9e-f24889245a5d" }
        }
        function "Watch Movie on Private Screen" {
            id: "aeafffae-c0e8-45a3-9ad1-105825574ced"
            description: "This function represents the fact that the passenger can watch a movie/video on a private screen (i.e. the in-seat TV)."
            realizes: "880520c2-6bd9-4481-b9da-6ccee7b9d665"
            parent: "cf623512-1614-4548-9b4c-4c0e614c4516"
            port in "FIP1" { id: "d73de0eb-4983-46ed-bd95-5da115e73afd" }
        }
    }
    component "Cabin Crew" {
        id: "94f479d0-c672-4a9c-bfbb-1fb5e01a73ee"
        actor: true
        function "Command Airline-Imposed Video Broadcast" {
            id: "37c0f280-8012-4736-82df-4e9114fc61d4"
            description: "This function consists in triggering the play of pre-recorded airline videos. In particular, these imposed videos inclure pre-flight safety instructions (detailed explanation given before takeoff to airline passengers about the safety features of a commercial aircraft). Such imposed videos can also be commercial ads."
            realizes: "cfde8bb1-4df9-4573-a516-41df984ac753"
            parent: "bca11666-f58a-41e4-9a16-5964771b55b0"
            port in "FIP 1" { id: "e4803d1c-acf1-4636-a2a6-596bd8cae78e" }
            port in "FIP 2" { id: "0bffc6a4-3a9b-404e-9b17-86da28903a3e" }
            port in "FIP 3" { id: "7e9d8a6b-7bb6-47c2-8f29-7237dc306243" }
            port out "FOP 1" { id: "f2d30d4d-56a8-4d65-be3a-ed1ffd82447f" }
            port out "FOP 2" { id: "2ce011ab-c48e-4a04-a14e-5051f5ac1a1d" }
            port out "FOP 3" { id: "09867250-647d-463f-ab1a-c18abb9ac9d4" }
            port out "FOP 4" { id: "ab15ab40-c207-4e07-9598-66bcf61cce5b" }
        }
        function "Launch Pre-Flight Tests and Analyze Results" {
            id: "82445ac7-1b27-4c0c-bdff-7fda4e1e7496"
            description: "This function consists in triggering a series of system auto-tests before each flight."
            realizes: "64847aed-d627-4812-a2de-a5dd8d7ce573"
            parent: "bca11666-f58a-41e4-9a16-5964771b55b0"
        }
    }
    component "Aircraft" {
        id: "a0ceff37-72a6-4da7-a646-89425a748235"
        actor: true
        port inout "CP 1" { id: "86e93a9f-0c86-42e1-bead-f3dfd12a0020" }
        port out "CP 2" { id: "f9a5f664-0853-4704-8ed6-2cf7fc0cf47e" }
        function "Play Audio on Cabin Aircraft Speakers" {
            id: "c5e641ab-a062-4856-87ae-e675061c3eb5"
            description: "This function receives digital audio streams and play them on the aricraft speakers."
            realizes: "6b9e59a9-89f2-497e-a348-3aef80689e61"
            parent: "cd4b1bad-e12e-45eb-96fd-dfec5b483457"
            port in "FIP 1" { id: "f5eef474-f4e0-4de2-bbd2-feeb9a3275c8" }
            port out "FOP1" { id: "34180189-9aaf-4c20-a861-49999b5bf55d" }
        }
        function "Provide Electrical Power" {
            id: "4d1c1e5c-4c96-41fc-a4a5-4bd74085f320"
            description: "This function is responsible for providing power consumption profiles. According to different flying conditions and phases, the power made available to the IFE system may differ."
            realizes: "135e77a7-79ef-45e8-b138-67e283a14cdd"
            parent: "cd4b1bad-e12e-45eb-96fd-dfec5b483457"
            port out "FOP 1" { id: "c16e6c8f-a0b0-4cb3-b748-f2785b450814" }
        }
        function "Provide Exterior-View Videos" {
            id: "9ae12cc6-8bb4-4f99-ad87-8a30a22b919b"
            description: "This functions relays the videos captured external cameras, typically located on the tail or the nose of the plane."
            realizes: "dacbebb6-72c7-4d4c-9009-aa95eba16dd3"
            parent: "cd4b1bad-e12e-45eb-96fd-dfec5b483457"
        }
        function "Provide Navigation Data" {
            id: "ebce47df-f06c-4eea-b367-b828df4b44fa"
            description: "Navigation is the determination of position and direction on or above the surface of the Earth. Avionics can use satellite-based systems (such as GPS and WAAS), ground-based systems (such as VOR or LORAN), or any combination thereof. The IFE system requires this data to run the moving-map service."
            realizes: "6ce8f26a-ef90-4420-94e1-acf6b2393720"
            parent: "cd4b1bad-e12e-45eb-96fd-dfec5b483457"
        }
        function "Provide Satellite Communication Means" {
            id: "55e3b1b8-dea2-4fbf-b78f-0f270009b56a"
            description: "This function consists in providing satellite network, typically for telephony or in-cabin wifi systems."
            realizes: "82ab0ab9-2c03-4689-989a-530aae6fcf94"
            parent: "cd4b1bad-e12e-45eb-96fd-dfec5b483457"
        }
        function "Send Audio Announcements" {
            id: "c6ed6258-be4c-4784-a6e4-3e95f9f22dc7"
            description: "Audio announcements include: boarding, pre-flight announcements, captain's announcements, safety briefing, turbulence, take-off/ascent, descent/final, landing, etc."
            realizes: "8f279b07-26aa-46f6-8c9a-2a132f0b1177"
            parent: "cd4b1bad-e12e-45eb-96fd-dfec5b483457"
            port out "FOP 5" { id: "fafaf529-2bce-48b2-b14c-78b23cbe485e" }
        }
        function "Send Decompression Notification" {
            id: "e01b39c5-f018-4d9c-af36-c83f1b9a8a81"
            description: "If ever a decompression occurs, a notification is sent to the IFE system (which will automatically swtich to \"Halted\" mode."
            realizes: "9972b7a8-df8a-4626-b298-1dd0d19e451e"
            parent: "cd4b1bad-e12e-45eb-96fd-dfec5b483457"
            port out "FOP 1" { id: "08d1a082-bf7f-41ac-bd51-e8f1201bf7f9" }
        }
    }
    component "Ground Operator" {
        id: "d459ccb1-cad9-4756-bb0c-b151c60461e7"
        actor: true
        function "Configure the System" {
            id: "9307409a-5b76-4a96-876c-85468c60e376"
            description: "[Not covered in this version of the model]"
            realizes: "88faafec-6848-4494-8dd3-2acda3e53688"
            parent: "bf6388c4-3549-46a6-af08-48412581967e"
        }
        function "Launch Tests and Analyze Results" {
            id: "05aefe2e-d873-4fb0-8ad7-4dafc46c764e"
            description: "[Not covered in the version of the model] This function consists in triggering system tests, typically during maintenance phases."
            realizes: "ff70c856-f667-4e61-98d7-00190703d53a"
            parent: "bf6388c4-3549-46a6-af08-48412581967e"
        }
        function "Set Passenger Service Authorization" {
            id: "9d3d6fe7-16df-4382-9d49-492e4cd2b73f"
            description: "The airline company may have specific business strategies: According to the class they are travelling in, passengers may benefit from different IFE services. This function consists in uploading a profile that describes the authorization schemes."
            realizes: "5e0fff47-a1a5-4a13-867c-18348bd894af"
            parent: "bf6388c4-3549-46a6-af08-48412581967e"
            port out "FOP 1" { id: "dce66cba-3052-487c-833c-99c441efbdf0" }
        }
        function "Update Media Content" {
            id: "d5254d45-3946-431f-8162-a28a3f2263a7"
            description: "This function consists in updating the IFE media server with new content. Using a specific device, the ground operator can for example updload new movies/music and remove others."
            realizes: "4d0046b4-4865-438d-b09b-fd95671db09b"
            parent: "bf6388c4-3549-46a6-af08-48412581967e"
            port out "FOP 1" { id: "d98eee35-a6ff-40b3-a54e-f9b56e7c8236" }
        }
    }
    function "Entertain with IFE System" {
        id: "cf623512-1614-4548-9b4c-4c0e614c4516"
        description: "This function is a high-level function gathering all the what is performed by the passenger actor."
        realizes: "3688cf59-97ee-4446-8041-7941cee6c767"
    }
    function "IFE System (2)" {
        id: "145e28dc-9fbe-4f01-a378-5af8cbe8e2d7"
        description: "This function is a high-level function gathering all the what is performed by the IFE system."
        realizes: "6d10ccfa-1be8-4ab6-a0d8-74941051595b"
        function "Manage Audio and Video Diffusion" {
            id: "95495bbc-987c-4b17-95d0-9aa5a87f0344"
            description: "This function is responsible for broadcasting audio and video streams, should they be VOD movies, imposed videos, on-demand or imposed music. This function also covers the transmission of audio announcements (coming from the aircraft) and the IFE passenger service interruptions triggered by these announcements."
            realizes: "ba4d638d-e6bb-4908-94f4-eb974405c733"
            function "Broadcast Stored Audio and Video" {
                id: "d4ee89c8-dcf8-4fa5-b276-0afa03d409c9"
                description: "This function is responsible for broadcasting existing audio and video media. Transmitting audio and video streams means: - Sending them to each passenger (on Seat TVs) - Sending them to the cabin screens"
                realizes: "e1808524-dfa9-4883-a809-c44a87712d1b"
            }
            function "Display Video and Play Audio" {
                id: "cbe07842-a67d-488a-bf29-5b46d3369c15"
                description: "This function aggregates different display and media playing functions, which are typically allocated to different equiments: Seat TVs, Cabin Screens, etc."
            }
            function "Process Audio Video Requests" {
                id: "a8452b58-3135-457b-8d85-59bd23240297"
                description: "This function groups sub functions that are responsible for tranforming the end-user (passenger, cabin crew) selections in applicative commands."
            }
        }
        function "Manage Passenger Services Lifecycle" {
            id: "11f3e0e8-a78d-4962-bed8-9672b1167331"
            description: "This function is responsible for controling the launch of services such as VOD, gaming, moving-map, etc. It determines the availability of the services given the airline company authorization scheme and the constraints received from the aircraft. This function provides means to select among a predefined list of services."
            realizes: "8a4b7443-4f73-49c2-a175-940a5d8accea"
        }
        function "Provide Access to Digital Media" {
            id: "2a6d57b4-97f3-4344-b645-cefcc9462708"
            description: "This functions gathers the functions related to the management of the digital media content of the IFE: load and storage."
            realizes: "6b54b12a-50f8-4ce5-b935-08edbf655d1f"
        }
        function "Provide Aircraft Interface" {
            id: "c268e3f1-3cbc-4e62-8ae2-75975bc4481f"
            description: "This functions regroups all the functions that interact directly with the aircraft."
        }
        function "Run Services" {
            id: "355a6d2b-8c53-46f8-8c0b-38356b4d4271"
            description: "This function regroups all IFE services running functions."
            realizes: "0bfc000c-dd38-40d3-9cd3-2037905a7ab3"
            function "Run Cabin Intercommunication Service" {
                id: "81d017bc-85ca-4bcd-acc9-51423542c61e"
                description: "Cabin intercommunication is a cabin crew service. This function gathers all functions in charge of managing the interactions of the cabin crew related to imposed videos (selection of videos, play/pause/stop controls, display of video data, etc.)."
                realizes: "7b7b0d2c-27e8-4ca0-8a5f-2790a37e97b6"
            }
            function "Run Video-On-Demand Service" {
                id: "7a8c51b2-c42f-49cd-a003-752d0de0b4da"
                description: "VOD Service is a passenger service. This function gathers all functions in charge of managing the VOD-related interactions of the passenger (selection of movies, play/pause/stop controls, display of movie data, etc.)."
                realizes: "26cd71b0-ec92-41a9-b9fb-6878beede810"
            }
        }
    }
    function "Perform Cabin Management Activities" {
        id: "bca11666-f58a-41e4-9a16-5964771b55b0"
        description: "This function is a high-level function gathering all the what is performed by the ground operator actor."
        realizes: "3398fdbf-3415-4d03-a457-477324d2e091"
    }
    function "Perform Maintenance, Configuration and Test Operations" {
        id: "bf6388c4-3549-46a6-af08-48412581967e"
        description: "This function is a high-level function gathering all the what is performed by the aircraft actor."
        realizes: "cd62e8fb-8a89-4f33-bd0f-a2bb935a5e08"
    }
    function "Provide Aircraft Information, Commands and Means" {
        id: "cd4b1bad-e12e-45eb-96fd-dfec5b483457"
        description: "This function is a high-level function gathering all the what is performed by the aircraft actor."
        realizes: "7de3b04b-6291-4098-afbd-2fb72cd25022"
    }
    functional_exchange "Decompression Notification" { id: "c385bac1-ff71-4f7c-b203-40e7779a63f8" from: "4b01ac5e-7920-491d-af35-1846163e5ad9.FOP 1" to: "1e812d3d-6c35-42c9-80fa-eb64dfc1a3a8.FIP 1" }
    functional_exchange "Power Consumption Profile" { id: "7b3d0795-e518-4b28-9804-cbb023e2640b" from: "4b01ac5e-7920-491d-af35-1846163e5ad9.FOP 2" to: "1e812d3d-6c35-42c9-80fa-eb64dfc1a3a8.FIP 2" }
    functional_exchange "Audio Stream" { id: "83a3c9ae-1cb3-4cab-885f-de529d4585ea" from: "ee920873-7e2d-487f-a51e-76236d6b094e.FOP 1" to: "39f96e89-8ead-4742-bc1f-6eb0dd33990b.FIP 1" }
    functional_exchange "Digital Media" { id: "1070ab6d-6e51-4aa9-8950-c3d49c0ca727" from: "70d62f70-d0b6-491c-9e9e-f5ed77179ce4.FOP 1" to: "be9b5663-23ea-4413-abeb-3d5df84c93b8.FIP 4" }
    functional_exchange "VOD Movie Control Command" { id: "df26fc8b-2837-40a0-ac8a-96ad5c39f9ac" from: "faf3223f-3f47-4ad6-9c58-353b34e1a30f.FOP 2" to: "27d03649-545b-40b0-8407-721165539005.FIP 2" exchange_items: ["1885aa4c-b56f-48cb-a433-a135f0bd7b01"] category: "VOD Movie Control Commands" }
    functional_exchange "Imposed Video Control  Command" { id: "127898d1-536b-4fbe-a630-5f0109d15cea" from: "7069fd31-6eb0-49fa-ae52-0b78307c2748.FOP 3" to: "27d03649-545b-40b0-8407-721165539005.FIP 3" exchange_items: ["ba2b52d2-9f7a-4f53-9e44-24c00af74a53"] }
    functional_exchange "Broadcast Data" { id: "4d67efdd-6484-4d84-a242-6b0210eec6f7" from: "27d03649-545b-40b0-8407-721165539005.FOP 1" to: "61bf6852-3f28-425d-b8ed-4d38b2b4eebb.FIP 2" exchange_items: ["a86aeed3-084c-4618-b23a-bd385f92ecb0"] }
    functional_exchange "Imposed Video Status" { id: "4bba771e-a9df-467f-acde-0929d06e0d95" from: "b55db2ae-a407-4c31-8093-75f0841094eb.FOP 1" to: "566f8900-7408-42d3-82f8-ba18bf0d923f.FIP 1" }
    functional_exchange "Audio-Video Stream" { id: "845007fe-fe45-46ec-a109-47742ac52529" from: "61bf6852-3f28-425d-b8ed-4d38b2b4eebb.FOP 1" to: "7b604e03-85be-4d93-8035-8a84ef8086db.FIP 1" exchange_items: ["f1309e1f-e30b-47ed-8027-41f12017ef50"] }
    functional_exchange "Video Stream" { id: "99801e2d-2ff3-4e61-bd8b-5fb26601bf96" from: "61bf6852-3f28-425d-b8ed-4d38b2b4eebb.FOP 2" to: "2632b523-b577-48c5-afba-3eb4ccde3254.FIP 1" }
    functional_exchange "Pause-Stop Notification" { id: "2eb58ffb-766e-450d-b436-5d1228203493" from: "27d03649-545b-40b0-8407-721165539005.FOP 2" to: "7b604e03-85be-4d93-8035-8a84ef8086db.FIP 2" }
    functional_exchange "Pause-Stop Notification (2)" { id: "472103e7-6717-4c09-96dc-a9b4f4b3f221" from: "27d03649-545b-40b0-8407-721165539005.FOP 2" to: "2632b523-b577-48c5-afba-3eb4ccde3254.FIP 3" }
    functional_exchange "Pause-Stop Notification (3)" { id: "f0ae5ac4-8841-4866-b3db-7377f0891da6" from: "27d03649-545b-40b0-8407-721165539005.FOP 2" to: "b55db2ae-a407-4c31-8093-75f0841094eb.FIP 4" }
    functional_exchange "Imposed Video Playing Status" { id: "6fd71d7f-b493-4678-87d1-515eccf54213" from: "27d03649-545b-40b0-8407-721165539005.FOP 4" to: "b55db2ae-a407-4c31-8093-75f0841094eb.FIP 1" }
    functional_exchange "Audio Stream (2)" { id: "cfbfc6da-1898-4a5d-8a93-84865d97a1b3" from: "39f96e89-8ead-4742-bc1f-6eb0dd33990b.FOP 3" to: "7b604e03-85be-4d93-8035-8a84ef8086db.FIP 3" }
    functional_exchange "VOD Movie Data Request" { id: "816d9b3e-7345-4b2b-b67f-1628d8034357" from: "5165a225-e662-4b4a-95cd-c94fc5726524.FOP 3" to: "651179c3-78b6-4a6f-a174-74405b81b36a.FIP 1" category: "Media Requests" }
    functional_exchange "VOD Movie Description Request" { id: "7738da4d-7ccc-4eb6-973b-c28c3319d97a" from: "5165a225-e662-4b4a-95cd-c94fc5726524.FOP 4" to: "651179c3-78b6-4a6f-a174-74405b81b36a.FIP 2" category: "Media Requests" }
    functional_exchange "VOD Movie List Request" { id: "c111592c-86f1-43ab-9979-eb6dc669c9a4" from: "5165a225-e662-4b4a-95cd-c94fc5726524.FOP 5" to: "651179c3-78b6-4a6f-a174-74405b81b36a.FIP 3" category: "Media Requests" }
    functional_exchange "Available VOD Movies List" { id: "27751fcd-c868-4b0b-8de7-2c819e912edf" from: "651179c3-78b6-4a6f-a174-74405b81b36a.FOP 1" to: "94aebe8b-af13-4a7a-a7ee-8fd10a7fe489.FIP 4" category: "Media Content" }
    functional_exchange "Movie Description" { id: "bbabfba6-7fbf-47e0-a5db-d14b1c72f798" from: "651179c3-78b6-4a6f-a174-74405b81b36a.FOP 2" to: "94aebe8b-af13-4a7a-a7ee-8fd10a7fe489.FIP 1" category: "Media Content" }
    functional_exchange "Movie Cover" { id: "ff081aeb-7c3e-4778-b8ab-6d1c3718aab9" from: "651179c3-78b6-4a6f-a174-74405b81b36a.FOP 3" to: "94aebe8b-af13-4a7a-a7ee-8fd10a7fe489.FIP 3" category: "Media Content" }
    functional_exchange "Available Languages List" { id: "8e23a789-19bb-4c39-85ab-ada245047400" from: "651179c3-78b6-4a6f-a174-74405b81b36a.FOP 4" to: "94aebe8b-af13-4a7a-a7ee-8fd10a7fe489.FIP 2" category: "Media Content" }
    functional_exchange "VOD Movie List Request (2)" { id: "793b6f83-aadc-4b2e-804d-26190aab3e1d" from: "910a61f1-3da0-404f-9db5-c226af9a3c47.FOP 2" to: "651179c3-78b6-4a6f-a174-74405b81b36a.FIP 8" }
    functional_exchange "VOD Movie Interruption Command" { id: "4aee9cf0-9b14-436e-86ad-5a11d4866e40" from: "910a61f1-3da0-404f-9db5-c226af9a3c47.FOP 3" to: "3136218c-bf14-437d-99c9-6593352eead9.FIP 2" }
    functional_exchange "Imposed Video Selection Command" { id: "530f8d5b-90c2-4787-ba1e-676ba7937c69" from: "1aa42ce5-1c2c-4473-9840-7d20a60c377e.FOP 2" to: "0d727549-f01f-4cc6-ae27-5f25220ece06.FIP 1" }
    functional_exchange "Available Imposed Video List" { id: "98cb1109-6bb4-4592-bf25-99bdc93db83a" from: "0d727549-f01f-4cc6-ae27-5f25220ece06.FOP2" to: "a90e98e8-741e-478a-9e14-af7a50ee2525.FIP1" category: "Media Content" }
    functional_exchange "Passenger Homepage Display Command" { id: "ac9e88f4-8781-42b0-bb44-c8d725c39b92" from: "2ea2703a-6dd1-4835-aaae-16179864ac1d.FOP 1" to: "70f1acf3-a32d-4190-b97e-3eb361a559e7.FIP 1" }
    functional_exchange "VOD Service Activation Command" { id: "cbc21308-951f-4d40-b2e8-50d8816a1d5d" from: "2ea2703a-6dd1-4835-aaae-16179864ac1d.FOP1" to: "c3b17f24-ae52-413f-ace7-6baab1287b02.FIP1" }
    functional_exchange "Available Passenger Service Request" { id: "c2d17c2a-ff11-417e-989f-48b54d4388a4" from: "70f1acf3-a32d-4190-b97e-3eb361a559e7.FOP 2" to: "5a58fd72-1b57-41dc-8b7b-b0b8cf577236.FIP 3" }
    functional_exchange "Available Passenger Services List" { id: "31bc5010-decb-4d62-96eb-1628496778de" from: "5a58fd72-1b57-41dc-8b7b-b0b8cf577236.FOP 1" to: "70f1acf3-a32d-4190-b97e-3eb361a559e7.FIP 2" }
    functional_exchange "Available Passenger Services List (2)" { id: "eb002c8e-031e-4129-8a22-a4f175353b9e" from: "5a58fd72-1b57-41dc-8b7b-b0b8cf577236.FOP 1" to: "c3b17f24-ae52-413f-ace7-6baab1287b02.FIP 4" }
    functional_exchange "Service Status Request" { id: "cc53c968-ebe2-4a64-a257-e1647ce7b1e2" from: "c3b17f24-ae52-413f-ace7-6baab1287b02.FOP 1" to: "3e9ccc30-596e-40e6-8941-64838248dcee.FIP 1" }
    functional_exchange "Service Stored State" { id: "2cad898f-4c0b-4956-a24f-b86e257272c0" from: "3e9ccc30-596e-40e6-8941-64838248dcee.FOP 3" to: "c3b17f24-ae52-413f-ace7-6baab1287b02.FIP 2" }
    functional_exchange "Service Deactivation Command" { id: "8212d143-bc97-4152-8351-646edc65a4c9" from: "2ea2703a-6dd1-4835-aaae-16179864ac1d.FOP 2" to: "3e9ccc30-596e-40e6-8941-64838248dcee.FIP 2" }
    functional_exchange "Service Interruption Command" { id: "7b1d7bc7-ec98-4f52-a99c-418a36980d63" from: "7069fd31-6eb0-49fa-ae52-0b78307c2748.FOP 4" to: "b4623d17-6e62-411d-839c-2d768664a920.FIP 3" category: "Service Activation / Interruption" }
    functional_exchange "Service Interruption Command (2)" { id: "fc741435-84fa-457c-b116-13d8c0f15249" from: "b4623d17-6e62-411d-839c-2d768664a920.FOP1" to: "910a61f1-3da0-404f-9db5-c226af9a3c47.FIP 2" category: "Service Activation / Interruption" }
    functional_exchange "Passenger VOD Service Activation" { id: "89c75525-66d8-4da4-a704-9641017c30d1" from: "c3b17f24-ae52-413f-ace7-6baab1287b02.FOP 4" to: "910a61f1-3da0-404f-9db5-c226af9a3c47.FIP 1" category: "Service Activation / Interruption" }
    functional_exchange "VOD Movie Status" { id: "544601f7-fbc3-4077-918c-b9199e7452e1" from: "3136218c-bf14-437d-99c9-6593352eead9.FOP 3" to: "910a61f1-3da0-404f-9db5-c226af9a3c47.FIP 3" }
    functional_exchange "IFE Operating Profile" { id: "8c966c78-51fb-4436-b36e-cc857783924d" from: "1e812d3d-6c35-42c9-80fa-eb64dfc1a3a8.FOP 1" to: "5a58fd72-1b57-41dc-8b7b-b0b8cf577236.FIP 2" }
    functional_exchange "Service Interruption Command (3)" { id: "9a9a0ba6-1725-4026-92da-dcb18e1b0f4e" from: "39f96e89-8ead-4742-bc1f-6eb0dd33990b.FOP 4" to: "b4623d17-6e62-411d-839c-2d768664a920.FIP 2" category: "Service Activation / Interruption" }
    functional_exchange "Audio Interruption Notification" { id: "b00d3b7f-56ac-4491-921e-fe1785c1c228" from: "b4623d17-6e62-411d-839c-2d768664a920.FOP 2" to: "4ff700ef-4f6b-4151-917b-e59064e5a225.FIP 1" category: "Service Activation / Interruption" }
    functional_exchange "Audio Stream (3)" { id: "223cf5ec-0222-4e32-9646-71bbc85cdd70" from: "61bf6852-3f28-425d-b8ed-4d38b2b4eebb.FOP 3" to: "30d03eaa-2615-481c-bf1f-53f177d8147c.FIP 1" }
    functional_exchange "Displayed Imposed Video Description" { id: "a1b30786-768a-4789-a8c7-030dfd544d69" from: "a90e98e8-741e-478a-9e14-af7a50ee2525.FOP2" to: "37c0f280-8012-4736-82df-4e9114fc61d4.FIP 1" category: "Displayed Imposed Video Data" }
    functional_exchange "Played Audio Video" { id: "654aacca-49b2-49fc-949c-ed6a871c6e42" from: "7b604e03-85be-4d93-8035-8a84ef8086db.FOP 1" to: "aeafffae-c0e8-45a3-9ad1-105825574ced.FIP1" category: "Played Media" }
    functional_exchange "VOD Service Selection" { id: "77bbd9a8-d804-4b57-aab6-77bd6c5add42" from: "63136eb0-622b-43cc-b4da-55b86ae281c1.FOP1" to: "2ea2703a-6dd1-4835-aaae-16179864ac1d.FIP1" category: "Passenger Service Selection" }
    functional_exchange "VOD Movie Play Selection" { id: "becb1a69-c67a-40b4-8d9e-2ff2c0f6761e" from: "5165a225-e662-4b4a-95cd-c94fc5726524.FOP 6" to: "faf3223f-3f47-4ad6-9c58-353b34e1a30f.FIP 3" exchange_items: ["e3830e92-47f9-4a57-b9ac-708bc64f6937"] category: "VOD Movie Control Commands" }
    functional_exchange "Media Files" { id: "3b5f13a1-3d95-4fa5-95e8-4724c2e4a063" from: "be9b5663-23ea-4413-abeb-3d5df84c93b8.FOP 1" to: "27d03649-545b-40b0-8407-721165539005.FIP 1" exchange_items: ["1fdba7f2-be01-4882-b999-b65528c3ee35"] category: "Media Content" }
    functional_exchange "Displayed Movies List" { id: "6f60db60-8fc5-427f-9b15-fc4a6373d642" from: "94aebe8b-af13-4a7a-a7ee-8fd10a7fe489.FOP 4" to: "bcb08500-a828-43b0-ac8b-ff4c7cb3b6b4.FIP 2" category: "Displayed VOD Movie Data" }
    functional_exchange "Audio Signal" { id: "589697fe-d350-452c-9afa-5cdd98027e5e" from: "c5e641ab-a062-4856-87ae-e675061c3eb5.FOP1" to: "295db719-853a-4cb6-9bbe-2e66b5ae9ace.FIP3" }
    functional_exchange "Passenger Service Authorization Profile" { id: "68708a1d-448b-483f-96cd-4478cc3ff088" from: "9d3d6fe7-16df-4382-9d49-492e4cd2b73f.FOP 1" to: "5a58fd72-1b57-41dc-8b7b-b0b8cf577236.FIP 1" }
    functional_exchange "Displayed Passenger Services List" { id: "02a756c6-5f3e-41aa-80ee-b6f7049b959f" from: "70f1acf3-a32d-4190-b97e-3eb361a559e7.FOP1" to: "63136eb0-622b-43cc-b4da-55b86ae281c1.FIP1" }
    functional_exchange "Displayed Available Languages" { id: "6265c622-964e-4caa-bf7d-49a2d4b48ad0" from: "94aebe8b-af13-4a7a-a7ee-8fd10a7fe489.FOP 2" to: "bcb08500-a828-43b0-ac8b-ff4c7cb3b6b4.FIP 1" category: "Displayed VOD Movie Data" }
    functional_exchange "Displayed Movie Description" { id: "07d33b69-6881-40bd-9573-0f2d3e790544" from: "94aebe8b-af13-4a7a-a7ee-8fd10a7fe489.FOP 1" to: "bcb08500-a828-43b0-ac8b-ff4c7cb3b6b4.FIP 3" category: "Displayed VOD Movie Data" }
    functional_exchange "VOD Movie Stop Selection" { id: "f4478adf-d442-4e3b-a3bc-bc108f92957e" from: "5165a225-e662-4b4a-95cd-c94fc5726524.FOP 7" to: "faf3223f-3f47-4ad6-9c58-353b34e1a30f.FIP 2" category: "VOD Movie Control Commands" }
    functional_exchange "Media IDs" { id: "bd658ea3-c425-4844-9ba1-31131e38d0dc" from: "27d03649-545b-40b0-8407-721165539005.FOP 3" to: "be9b5663-23ea-4413-abeb-3d5df84c93b8.FIP 1" category: "Media Requests" }
    functional_exchange "Media Content Request" { id: "9d22d9ef-e662-40d2-afab-1ee76f49b4af" from: "0d727549-f01f-4cc6-ae27-5f25220ece06.FOP 2" to: "be9b5663-23ea-4413-abeb-3d5df84c93b8.FIP 2" category: "Media Requests" }
    functional_exchange "Imposed Video Data" { id: "55b24053-decd-4e39-9fe6-dced898b454f" from: "be9b5663-23ea-4413-abeb-3d5df84c93b8.FOP 4" to: "0d727549-f01f-4cc6-ae27-5f25220ece06.FIP 2" category: "Media Content" }
    functional_exchange "Media Content Request (2)" { id: "a2bbd112-1fc6-40aa-baa7-6e8e5f645084" from: "651179c3-78b6-4a6f-a174-74405b81b36a.FOP 5" to: "be9b5663-23ea-4413-abeb-3d5df84c93b8.FIP 3" category: "Media Requests" }
    functional_exchange "VOD Movies List" { id: "70738147-7b2b-4aa0-a87b-f4942fd00998" from: "be9b5663-23ea-4413-abeb-3d5df84c93b8.FOP 2" to: "651179c3-78b6-4a6f-a174-74405b81b36a.FIP 4" category: "Media Content" }
    functional_exchange "VOD Movie Data" { id: "f1f7e78f-7745-4687-a89b-229520c41a8d" from: "be9b5663-23ea-4413-abeb-3d5df84c93b8.FOP 3" to: "651179c3-78b6-4a6f-a174-74405b81b36a.FIP 5" category: "Media Content" }
    functional_exchange "Displayed Imposed Videos List" { id: "b78c85ec-d3c9-43f0-a006-aa5340599509" from: "a90e98e8-741e-478a-9e14-af7a50ee2525.FOP1" to: "37c0f280-8012-4736-82df-4e9114fc61d4.FIP 2" category: "Displayed Imposed Video Data" }
    functional_exchange "VOD Movie Language Selection Command" { id: "42c74c0c-9903-4d3c-9372-bdbaf742257b" from: "5165a225-e662-4b4a-95cd-c94fc5726524.FOP 1" to: "651179c3-78b6-4a6f-a174-74405b81b36a.FIP 6" }
    functional_exchange "VOD Movie Selection Command" { id: "a565c017-0a94-4788-baf4-835bf3d03205" from: "5165a225-e662-4b4a-95cd-c94fc5726524.FOP 2" to: "651179c3-78b6-4a6f-a174-74405b81b36a.FIP 7" }
    functional_exchange "Digital Media (2)" { id: "0e1ff1e8-00c9-4cb9-8d96-7cdc2ebcf735" from: "d5254d45-3946-431f-8162-a28a3f2263a7.FOP 1" to: "70d62f70-d0b6-491c-9e9e-f5ed77179ce4.FIP 1" }
    functional_exchange "Played Video" { id: "86d0f636-e27f-436d-9e58-0ce047bd6cdd" from: "2632b523-b577-48c5-afba-3eb4ccde3254.FOP1" to: "295db719-853a-4cb6-9bbe-2e66b5ae9ace.FIP2" category: "Played Media" }
    functional_exchange "Audio Stream (4)" { id: "0ad613f5-c36e-4cb8-b9f9-ce38b8078929" from: "c6ed6258-be4c-4784-a6e4-3e95f9f22dc7.FOP 5" to: "ee920873-7e2d-487f-a51e-76236d6b094e.FIP 1" }
    functional_exchange "Audio Signal (2)" { id: "6e91454a-ca43-4d1e-b8f8-22435ec9c59a" from: "c5e641ab-a062-4856-87ae-e675061c3eb5.FOP1" to: "18da001e-dd8b-4041-ba4c-eb2211c624c4.FIP 1" }
    functional_exchange "VOD Movie Stop Selection (2)" { id: "1f6c53f8-243d-4981-b1f7-37c597cde3d3" from: "bcb08500-a828-43b0-ac8b-ff4c7cb3b6b4.FOP 1" to: "5165a225-e662-4b4a-95cd-c94fc5726524.FIP 1" category: "VOD Movie Control Commands" }
    functional_exchange "VOD Movie Play Selection (2)" { id: "91642f3d-3687-43e4-946c-71541aa590c5" from: "bcb08500-a828-43b0-ac8b-ff4c7cb3b6b4.FOP 2" to: "5165a225-e662-4b4a-95cd-c94fc5726524.FIP 2" category: "VOD Movie Control Commands" }
    functional_exchange "VOD Movie Pause Selection" { id: "7677133e-57ee-4827-8cbf-73ee1f55a5a4" from: "bcb08500-a828-43b0-ac8b-ff4c7cb3b6b4.FOP 3" to: "5165a225-e662-4b4a-95cd-c94fc5726524.FIP 3" category: "VOD Movie Control Commands" }
    functional_exchange "VOD Movie Language Selection" { id: "cdc4500f-0b03-4016-97eb-6094a12202aa" from: "bcb08500-a828-43b0-ac8b-ff4c7cb3b6b4.FOP 4" to: "5165a225-e662-4b4a-95cd-c94fc5726524.FIP 4" category: "VOD Selections" }
    functional_exchange "VOD Movie Selection" { id: "6d34ef5b-feba-4360-8812-382fe34c0bd3" from: "bcb08500-a828-43b0-ac8b-ff4c7cb3b6b4.FOP 5" to: "5165a225-e662-4b4a-95cd-c94fc5726524.FIP 5" category: "VOD Selections" }
    functional_exchange "VOD Movie Resume Selection" { id: "b80abaa1-09fb-4e6f-9033-84a8e7dd0dd7" from: "910a61f1-3da0-404f-9db5-c226af9a3c47.FOP 1" to: "faf3223f-3f47-4ad6-9c58-353b34e1a30f.FIP 1" category: "VOD Movie Control Commands" }
    functional_exchange "Displayed Imposed Video Playing Status" { id: "66ca912b-4dc1-4616-9a24-c24fef5ac11e" from: "566f8900-7408-42d3-82f8-ba18bf0d923f.FOP 1" to: "37c0f280-8012-4736-82df-4e9114fc61d4.FIP 3" }
    functional_exchange "Displayed Audio Interruption Screen" { id: "d8ae3120-7282-4314-bf35-41dd4c2e82f6" from: "4ff700ef-4f6b-4151-917b-e59064e5a225.FOP 1" to: "18da001e-dd8b-4041-ba4c-eb2211c624c4.FIP 2" category: "Played Media" }
    functional_exchange "Power Consumption Cap" { id: "8ab8e6a5-2c1f-4046-8aab-b44dd550b833" from: "4d1c1e5c-4c96-41fc-a4a5-4bd74085f320.FOP 1" to: "4b01ac5e-7920-491d-af35-1846163e5ad9.FIP 1" }
    functional_exchange "Decompression Notificaton" { id: "929f4bd6-ff1c-43d4-8361-abd7382f5d91" from: "e01b39c5-f018-4d9c-af36-c83f1b9a8a81.FOP 1" to: "4b01ac5e-7920-491d-af35-1846163e5ad9.FIP 2" }
    functional_exchange "Audio Stream (5)" { id: "9615df54-b339-4ff1-bd73-b16fb15ae35f" from: "30d03eaa-2615-481c-bf1f-53f177d8147c.FOP 1" to: "c5e641ab-a062-4856-87ae-e675061c3eb5.FIP 1" }
    functional_exchange "Imposed Video Play Selection" { id: "256a5004-f2cc-4c22-a4d4-715d4a775764" from: "37c0f280-8012-4736-82df-4e9114fc61d4.FOP 1" to: "1aa42ce5-1c2c-4473-9840-7d20a60c377e.FIP 1" category: "Imposed Video Control Commands" }
    functional_exchange "Imposed Video Pause Selection" { id: "6bce3c7b-9f42-439d-adcf-5e47055d355a" from: "37c0f280-8012-4736-82df-4e9114fc61d4.FOP 2" to: "1aa42ce5-1c2c-4473-9840-7d20a60c377e.FIP 2" category: "Imposed Video Control Commands" }
    functional_exchange "Imposed Video Stop Selection" { id: "8026ed14-01f5-42c4-8c69-d36afeee1075" from: "37c0f280-8012-4736-82df-4e9114fc61d4.FOP 3" to: "1aa42ce5-1c2c-4473-9840-7d20a60c377e.FIP 3" category: "Imposed Video Control Commands" }
    functional_exchange "Imposed Video List" { id: "f0fba710-c33e-43c7-8a55-a4f7cb82a17a" from: "be9b5663-23ea-4413-abeb-3d5df84c93b8.FOP 5" to: "0d727549-f01f-4cc6-ae27-5f25220ece06.FIP 3" category: "Media Content" }
    functional_exchange "Imposed Video Selection" { id: "0e7dae41-7386-402e-9e33-213a1bae69a2" from: "37c0f280-8012-4736-82df-4e9114fc61d4.FOP 4" to: "1aa42ce5-1c2c-4473-9840-7d20a60c377e.FIP 4" }
    functional_exchange "Imposed Video Play Selection (2)" { id: "bfd7c07a-c578-4064-841b-85663f2096ba" from: "1aa42ce5-1c2c-4473-9840-7d20a60c377e.FOP 3" to: "7069fd31-6eb0-49fa-ae52-0b78307c2748.FIP 2" category: "Imposed Video Control Commands" }
    functional_exchange "Imposed Video Pause Selection (2)" { id: "83421190-a4d9-4ae5-8281-3b8ca9ebeada" from: "1aa42ce5-1c2c-4473-9840-7d20a60c377e.FOP 4" to: "7069fd31-6eb0-49fa-ae52-0b78307c2748.FIP 3" category: "Imposed Video Control Commands" }
    functional_exchange "Imposed Video Stop Selection (2)" { id: "9431762e-b19e-4c66-aa71-3b1e681f3472" from: "1aa42ce5-1c2c-4473-9840-7d20a60c377e.FOP 5" to: "7069fd31-6eb0-49fa-ae52-0b78307c2748.FIP 4" category: "Imposed Video Control Commands" }
    functional_exchange "Audio Signal (3)" { id: "2c650bc2-d254-4026-b5cf-c2a001f289bd" from: "7b604e03-85be-4d93-8035-8a84ef8086db.FOP 3" to: "18da001e-dd8b-4041-ba4c-eb2211c624c4.FIP 3" category: "Played Media" }
    functional_exchange "Passenger Homepage Selection" { id: "4d2e526a-2166-4430-96e4-93e87b447b02" from: "63136eb0-622b-43cc-b4da-55b86ae281c1.FOP 2" to: "2ea2703a-6dd1-4835-aaae-16179864ac1d.FIP 1" category: "Passenger Service Selection" }
    component_exchange "Audio-Video Streams" { id: "254072d9-3ac4-4dc6-8261-21e5ecc12710" from_port: "_gzH_AI2cEd6Q36a6kYkaHQ.CP 3" to_port: "a0ceff37-72a6-4da7-a646-89425a748235.CP 1" }
    component_exchange "Aircraft Messages" { id: "8ae83bf8-69e8-4318-8771-fb8627b576f2" from_port: "a0ceff37-72a6-4da7-a646-89425a748235.CP 2" to_port: "_gzH_AI2cEd6Q36a6kYkaHQ.CP 4" }
    component_exchange "Media Data" { id: "d3ac3168-a54a-40e0-b517-9a154d41dc54" from_port: "_JNwHUJY7Ed6rH5SGn-UjSg.CP 1" to_port: "_neToII2dEd6Q36a6kYkaHQ.CP 3" }
    component_exchange "Streaming Protocol" { id: "fa8dcb53-6ff5-4237-868b-2396a9fbc47c" from_port: "_QHUAIJY7Ed6rH5SGn-UjSg.CP 1" to_port: "11e922f1-192b-43a5-9060-f935c26998a5.CP 1" }
    component_exchange "Service Management" { id: "1aa5b87b-7c2e-47d8-a88b-9f14c6ef5a92" from_port: "df043998-7fd1-4d4d-899f-7257788d644f.CP 1" to_port: "e02be1c5-a1a2-4c11-be30-259e92ef5826.CP 1" }
    component_exchange "Streaming Protocol (2)" { id: "f834b55d-a014-4f13-8bbb-3b7113a87439" from_port: "_QHUAIJY7Ed6rH5SGn-UjSg.CP 2" to_port: "_nxYVII2dEd6Q36a6kYkaHQ.CP 1" }
    component_exchange "Media Data (2)" { id: "bd61946c-3edf-4f99-a345-061febcafc3a" from_port: "_XnQJQJClEd6HJInKHsIvPg.CP 3" to_port: "_JNwHUJY7Ed6rH5SGn-UjSg.CP 2" }
    component_exchange "Command and Status" { id: "a3ddc9ab-45fe-4157-8db4-f355ad100eac" from_port: "_XnQJQJClEd6HJInKHsIvPg.CP 4" to_port: "_QHUAIJY7Ed6rH5SGn-UjSg.CP 3" }
    component_exchange "Service Management (2)" { id: "5af8d2ef-ad0f-4190-b71f-9096d9f7e5ea" from_port: "_XnQJQJClEd6HJInKHsIvPg.CP 5" to_port: "e02be1c5-a1a2-4c11-be30-259e92ef5826.CP 2" }
    component_exchange "Service Management (3)" { id: "ce4e7c76-a2fc-46aa-a7f4-1422ec8bdb1b" from_port: "df043998-7fd1-4d4d-899f-7257788d644f.CP 2" to_port: "_neToII2dEd6Q36a6kYkaHQ.CP 4" }
    component_exchange "Commands" { id: "f58a16a1-6663-4750-904e-0ba8249d58ea" from_port: "_QHUAIJY7Ed6rH5SGn-UjSg.CP 4" to_port: "_neToII2dEd6Q36a6kYkaHQ.CP 5" }
    component_exchange "Streaming Protocol (3)" { id: "96228350-a898-47ba-83c1-66abdbfecb36" from_port: "df043998-7fd1-4d4d-899f-7257788d644f.CP 3" to_port: "_nxYVII2dEd6Q36a6kYkaHQ.CP 2" }
    functional_chain "Watch Imposed Video on Cabin Screen" {
        id: "56f511e4-5d06-42d3-b96e-d5a2522f6bed"
        owner: "e373786f-d962-4a05-9360-902a524b7522"
        involves: ["37c0f280-8012-4736-82df-4e9114fc61d4", "1aa42ce5-1c2c-4473-9840-7d20a60c377e", "7069fd31-6eb0-49fa-ae52-0b78307c2748", "27d03649-545b-40b0-8407-721165539005", "be9b5663-23ea-4413-abeb-3d5df84c93b8", "27d03649-545b-40b0-8407-721165539005", "b55db2ae-a407-4c31-8093-75f0841094eb", "566f8900-7408-42d3-82f8-ba18bf0d923f", "37c0f280-8012-4736-82df-4e9114fc61d4", "61bf6852-3f28-425d-b8ed-4d38b2b4eebb", "30d03eaa-2615-481c-bf1f-53f177d8147c", "c5e641ab-a062-4856-87ae-e675061c3eb5", "295db719-853a-4cb6-9bbe-2e66b5ae9ace", "2632b523-b577-48c5-afba-3eb4ccde3254"]
    }
    functional_chain "Watch Imposed Video on Private Screen" {
        id: "8fe69f90-7472-4e05-ab57-1e0fa4e511a4"
        owner: "e373786f-d962-4a05-9360-902a524b7522"
        involves: ["37c0f280-8012-4736-82df-4e9114fc61d4", "1aa42ce5-1c2c-4473-9840-7d20a60c377e", "7069fd31-6eb0-49fa-ae52-0b78307c2748", "b4623d17-6e62-411d-839c-2d768664a920", "910a61f1-3da0-404f-9db5-c226af9a3c47", "3136218c-bf14-437d-99c9-6593352eead9", "27d03649-545b-40b0-8407-721165539005", "be9b5663-23ea-4413-abeb-3d5df84c93b8", "27d03649-545b-40b0-8407-721165539005", "61bf6852-3f28-425d-b8ed-4d38b2b4eebb", "7b604e03-85be-4d93-8035-8a84ef8086db", "aeafffae-c0e8-45a3-9ad1-105825574ced", "b55db2ae-a407-4c31-8093-75f0841094eb", "566f8900-7408-42d3-82f8-ba18bf0d923f", "37c0f280-8012-4736-82df-4e9114fc61d4"]
    }
    functional_chain "Broadcast Audio Announcement" {
        id: "849047d6-233a-4f85-bf60-8df1f145cda9"
        owner: "e373786f-d962-4a05-9360-902a524b7522"
        involves: ["c6ed6258-be4c-4784-a6e4-3e95f9f22dc7", "ee920873-7e2d-487f-a51e-76236d6b094e", "39f96e89-8ead-4742-bc1f-6eb0dd33990b", "7b604e03-85be-4d93-8035-8a84ef8086db", "18da001e-dd8b-4041-ba4c-eb2211c624c4", "b4623d17-6e62-411d-839c-2d768664a920", "4ff700ef-4f6b-4151-917b-e59064e5a225", "910a61f1-3da0-404f-9db5-c226af9a3c47", "3136218c-bf14-437d-99c9-6593352eead9"]
    }
    functional_chain "Stop Playing Imposed Video [NOT DONE]" {
        id: "3def4748-6a03-4e18-8004-0cacb9f3b457"
        owner: "e373786f-d962-4a05-9360-902a524b7522"
    }
    functional_chain "Pause Running Imposed Video [NOT DONE]" {
        id: "5eb59259-7021-4c20-805c-9115a89867a0"
        owner: "e373786f-d962-4a05-9360-902a524b7522"
    }
    functional_chain "Start Playing VOD Movie" {
        id: "24ca4fcd-9fe9-404c-8ce8-57dd25b038dc"
        owner: "58c038e4-77d4-4b0e-9a5d-7f4253285be2"
        involves: ["bcb08500-a828-43b0-ac8b-ff4c7cb3b6b4", "5165a225-e662-4b4a-95cd-c94fc5726524", "faf3223f-3f47-4ad6-9c58-353b34e1a30f", "27d03649-545b-40b0-8407-721165539005", "be9b5663-23ea-4413-abeb-3d5df84c93b8", "27d03649-545b-40b0-8407-721165539005", "61bf6852-3f28-425d-b8ed-4d38b2b4eebb", "7b604e03-85be-4d93-8035-8a84ef8086db", "aeafffae-c0e8-45a3-9ad1-105825574ced"]
    }
    functional_chain "Resume VOD Movie" {
        id: "9601e912-9538-4dde-9c4e-c631179e763c"
        owner: "58c038e4-77d4-4b0e-9a5d-7f4253285be2"
        involves: ["3136218c-bf14-437d-99c9-6593352eead9", "910a61f1-3da0-404f-9db5-c226af9a3c47", "faf3223f-3f47-4ad6-9c58-353b34e1a30f", "27d03649-545b-40b0-8407-721165539005", "be9b5663-23ea-4413-abeb-3d5df84c93b8", "27d03649-545b-40b0-8407-721165539005", "61bf6852-3f28-425d-b8ed-4d38b2b4eebb", "7b604e03-85be-4d93-8035-8a84ef8086db", "aeafffae-c0e8-45a3-9ad1-105825574ced"]
    }
    functional_chain "VOD Service Activation [NOT DONE]" {
        id: "e57ba3b5-fd92-4bf6-8b4d-1c7df1602a20"
        owner: "58c038e4-77d4-4b0e-9a5d-7f4253285be2"
    }
    functional_chain "VOD Movie Interruption [NOT DONE]" {
        id: "7cde2018-8a87-450f-934a-bca3ea150873"
        owner: "58c038e4-77d4-4b0e-9a5d-7f4253285be2"
    }
    functional_chain "Stop VOD Movie Play [NOT DONE]" {
        id: "1f37cd9c-1aa0-4698-95f9-c87437af0914"
        owner: "58c038e4-77d4-4b0e-9a5d-7f4253285be2"
    }
    capability_realization "Provide Audio and Video Intercommunication Means" {
        id: "e373786f-d962-4a05-9360-902a524b7522"
        involves: ["395ee0ab-cb3d-48e8-a411-09a98053292f", "94f479d0-c672-4a9c-bfbb-1fb5e01a73ee", "a0ceff37-72a6-4da7-a646-89425a748235", "_JNwHUJY7Ed6rH5SGn-UjSg", "_nxYVII2dEd6Q36a6kYkaHQ", "_QHUAIJY7Ed6rH5SGn-UjSg", "01a67cde-a09c-446e-9637-ab235664d23c", "_2SUXEJZFEd6rH5SGn-UjSg", "281c331f-1184-4c7d-afb2-7a55ff09a80f", "97e2a56c-0d4c-4a56-924a-63a31e6c50d7", "_SF1a8JY-Ed6rH5SGn-UjSg", "df043998-7fd1-4d4d-899f-7257788d644f", "e02be1c5-a1a2-4c11-be30-259e92ef5826", "11e922f1-192b-43a5-9060-f935c26998a5", "56f511e4-5d06-42d3-b96e-d5a2522f6bed", "5eb59259-7021-4c20-805c-9115a89867a0", "3def4748-6a03-4e18-8004-0cacb9f3b457", "849047d6-233a-4f85-bf60-8df1f145cda9", "8fe69f90-7472-4e05-ab57-1e0fa4e511a4"]
        realizes: "291fb362-9ae5-4779-8405-1281ea3709f6"
    }
    capability_realization "Provide Video Entertainment Services" {
        id: "58c038e4-77d4-4b0e-9a5d-7f4253285be2"
        description: "[Wikipedia] Video entertainment is provided via a large video screen at the front of a cabin section, as well as via smaller monitors situated every few rows above the aisles. Sound is supplied via the same headphones as those distributed for audio entertainment. Personal televisions (PTVs) for every passenger provide passengers with channels broadcasting new and classic films, as well as comedies, news, sports programming, documentaries, children's shows, and drama series. Some airlines also present news and current affairs programming, which are often pre-recorded and delivered in the early morning before flights commence. PTVs are operated via an In flight Management System which stores pre-recorded channels on a central server and streams them to PTV equipped seats during flight. VOD (Video On Demand) systems store individual programs separately, allowing a passenger to have a specific program streamed to them privately, and be able to control the playback."
        involves: ["395ee0ab-cb3d-48e8-a411-09a98053292f", "_JNwHUJY7Ed6rH5SGn-UjSg", "e02be1c5-a1a2-4c11-be30-259e92ef5826", "_2SUXEJZFEd6rH5SGn-UjSg", "281c331f-1184-4c7d-afb2-7a55ff09a80f", "_QHUAIJY7Ed6rH5SGn-UjSg", "_nxYVII2dEd6Q36a6kYkaHQ", "7cde2018-8a87-450f-934a-bca3ea150873", "9601e912-9538-4dde-9c4e-c631179e763c", "e57ba3b5-fd92-4bf6-8b4d-1c7df1602a20", "1f37cd9c-1aa0-4698-95f9-c87437af0914", "24ca4fcd-9fe9-404c-8ce8-57dd25b038dc"]
        realizes: "839ba873-8442-4a01-bd1a-1f4c3cc6ed52"
    }
    capability_realization "Integrate Aircraft Constraints" {
        id: "187c2207-0995-4241-b8a2-d45550185dff"
        involves: ["_SF1a8JY-Ed6rH5SGn-UjSg", "df043998-7fd1-4d4d-899f-7257788d644f", "_Rml6MJY-Ed6rH5SGn-UjSg"]
        realizes: "5acd248a-3961-4e71-85a4-5bc3b775215f"
    }
    capability_realization "Provide Access Management Control" {
        id: "4813aae0-442e-4b79-9d9f-cc2f236dbba4"
        involves: ["395ee0ab-cb3d-48e8-a411-09a98053292f", "df043998-7fd1-4d4d-899f-7257788d644f", "_2SUXEJZFEd6rH5SGn-UjSg", "281c331f-1184-4c7d-afb2-7a55ff09a80f", "_Rml6MJY-Ed6rH5SGn-UjSg", "e02be1c5-a1a2-4c11-be30-259e92ef5826"]
        realizes: "74e6e35e-b1e7-45c1-bd9f-5d80c0714472"
    }
    capability_realization "Provide Video Gaming Services" {
        id: "ae98c518-b305-41e6-b825-aa231d354325"
        description: "[Wikipedia] Video games are another emerging facet of in-flight entertainment. Some game systems are networked to allow interactive playing by multiple passengers. Later generations of IFE games began to shift focus from pure entertainment to learning. The best examples of this changing trend are the popular trivia game series and the Berlitz Word Traveler that allows passengers to learn a new language in their own language. Appearing as a mixture of lessons and mini games, passengers can learn the basics of a new language while being entertained"
        realizes: "b3a96943-f9e1-4249-97ce-c974ed889f41"
    }
    capability_realization "Provide Wi-Fi Connectivity Service" {
        id: "24c03d22-ddf6-4525-aa1b-b65e1360fe85"
        description: "[Wikipedia] In-flight internet service is provided either through a satellite network or an air-to-ground network. Data communication via satellite system allows passengers to connect to live internet from the individual IFE units or their laptops via the in-flight Wi-Fi access"
        realizes: "67bacc43-8066-4b96-8101-cf8f3c5f9b35"
    }
    capability_realization "Provide Satellite and Internal Telephony Services" {
        id: "5c8adc1c-079e-4cad-9b33-4dfb53db7f66"
        description: "[Wikipedia] Some airlines provide satellite telephones integrated into their system. These are either found at strategic locations in the aircraft or integrated into the passenger remote control used for the individual in-flight entertainment. Passengers can use their credit card to make phone calls anywhere on the ground. These systems are usually not capable of receiving incoming calls. There are also some aircraft that allow faxes to be sent and the rate is usually the same as the call rate, but at a per page rate. Some systems also allow the transmission of SMS. More modern systems allow passengers to call fellow passengers located in another seat by simply keying in the recipient's seat number."
        realizes: "43207098-1d88-4680-ae14-68f165e74c24"
    }
    capability_realization "Provide Moving-Map Services" {
        id: "9498f50c-65c6-428f-aeb3-a431c5f25e06"
        description: "[Wikipedia] A moving-map system is a real-time flight information video channel broadcast through to cabin project/video screens and personal televisions (PTVs). In addition to displaying a map that illustrates the position and direction of the plane, the system gives the altitude, airspeed, outside air temperature, distance to the destination, distance from the origination point, and local time. The moving-map system information is derived in real time from the aircraft's flight computer systems."
        realizes: "4e6fa11e-933f-4aec-b4c0-621c8fb244e5"
    }
    capability_realization "Provide Audio Entertainment Services" {
        id: "4c813481-a521-4f4b-904b-7a93569ef620"
        description: "[Wikipedia] Audio entertainment covers music, as well as news, information, and comedy. Most music channels are pre-recorded and feature their own DJs to provide chatter, song introductions, and interviews with artists. In addition, there is sometimes a channel devoted to the plane's radio communications, allowing passengers to listen in on the pilot's in-flight conversations with other planes and ground stations."
        realizes: "4cfe1222-e1e6-4af5-bcb7-b6a51b4ae50c"
    }
    capability_realization "Provide Personal Device Connectivity" {
        id: "dfb5ba2c-d16e-4cd6-921f-38a5265c939e"
        realizes: "9a942752-caaa-4a14-9e18-fca348022a41"
    }
    capability_realization "Provide Testing Interface" {
        id: "bd2a543a-d87b-4147-8ad4-7f77a110ced2"
        realizes: "1adec5b0-a145-4a8f-b6c3-35f0fe1ae2d5"
    }
    capability_realization "Provide Configuration Means" {
        id: "36429bbc-57fd-4862-b59e-003a11f4b146"
        realizes: "a643f8af-0104-49c8-add7-525806988ccb"
    }
    capability_realization "Update Entertainment Offer" {
        id: "b97f7931-82eb-48e0-a9e6-0d1f82b4235f"
        realizes: "32ff49c3-04bc-476e-b03e-bb6ecbcfe554"
    }
}

physical_architecture "Physical Architecture" {
    id: "9a0668cf-db26-4ee7-9339-c1d8658fa112"
    node "IFE System" {
        id: "5158c762-8f3c-4947-98f4-4cf085492bf5"
        behavior_component "BEHAVIOURAL PCs" {
            id: "c8114705-19fa-41b3-b534-a342f70c2e51"
            allocated_component: "724d6db9-9175-450b-8f05-023fcffec79c"
        }
        behavior_component "Aircraft Front Servers" {
            id: "d7856f65-ee48-4e3a-883a-71bcec485da1"
            allocated_component: "_8i1cwI2bEd6Q36a6kYkaHQ"
        }
        behavior_component "Aircraft Interface SW" {
            id: "f5da7965-ae21-404f-9407-74efd11234ed"
            allocated_component: "_gzH_AI2cEd6Q36a6kYkaHQ"
        }
        behavior_component "VSU SW" {
            id: "2bf4958b-9ec6-4427-9af0-1d5eefccd900"
        }
        behavior_component "ASU SW" {
            id: "6517229a-5dee-4817-978c-8f8cf1bfd3b5"
        }
        behavior_component "PVDU SW" {
            id: "8d1c0fbf-78f2-4cdf-ac2e-f14c31dd3506"
            allocated_component: "_8rBvcI2bEd6Q36a6kYkaHQ"
        }
        behavior_component "PVDU Interactions Manager" {
            id: "37d063af-0e66-4f9b-86fc-1a1aaec44b21"
            allocated_component: "_neToII2dEd6Q36a6kYkaHQ"
        }
        behavior_component "PVDU Airline-Specific Interactions Manager" {
            id: "9eef8f0f-1bf8-4e51-a17f-480b3afbfd85"
            port out "CP 2" { id: "c639fbd4-3740-4458-b7ad-14c14067c4a7" }
            port in "CP 1" { id: "c6ab41fe-02e1-4f03-88a8-b67ac7d0e02a" }
            allocated_component: "281c331f-1184-4c7d-afb2-7a55ff09a80f"
        }
        behavior_component "PVDU Video Player" {
            id: "e7049747-365c-4c2f-b769-f45221b9a05a"
            allocated_component: "_nxYVII2dEd6Q36a6kYkaHQ"
        }
        behavior_component "Cabin Video Unit SW" {
            id: "3cbac439-d663-452b-86b6-4f0c947a0047"
            allocated_component: "11e922f1-192b-43a5-9060-f935c26998a5"
        }
        behavior_component "Central Management Unit SW" {
            id: "fda6b8f9-bfe8-4c60-8443-9b8a9ee2eeeb"
            allocated_component: "8dbc153d-c9e8-4cb5-93af-ecf9a0fc939c"
        }
        behavior_component "CMU Interactions Manager" {
            id: "47cb2f3d-0ac5-4053-92d1-1af781f0ae20"
            allocated_component: "_XnQJQJClEd6HJInKHsIvPg"
        }
        behavior_component "Remote diagnostic equipment SW" {
            id: "95ea7294-402d-4783-b6e5-2e405c9f4ccb"
            function "PF1" {
                id: "88688938-d911-4317-b36e-266cef11a7f3"
                parent: "f46f4ddc-51cc-4b2c-ba6c-d9e8c0bae8ea"
                port out "FOP 1" { id: "5b950b88-688b-41b8-a4c1-b9f597332cca" }
            }
            function "PF2" {
                id: "fd6de739-ae93-4e5f-acaf-4e8fb8f0b2e5"
                parent: "f46f4ddc-51cc-4b2c-ba6c-d9e8c0bae8ea"
                port in "FIP 1" { id: "6e3f72e3-b50c-4ef7-9549-5541de878bc0" }
                port out "FOP 1" { id: "d0c971db-b83e-4c5f-a298-8f90989e62ce" }
                port out "FOP 2" { id: "2af3bfd5-708e-4a86-bff3-dc9589a2d76c" }
            }
            function "PF3.1" {
                id: "1c49d7b2-bb7a-4939-86c1-a73c0822fdf5"
                parent: "f46f4ddc-51cc-4b2c-ba6c-d9e8c0bae8ea"
                port in "FIP 1" { id: "a504f47c-d8ef-40da-bbeb-de4402d28d83" }
                port out "FOP 1" { id: "2a59f6f9-556f-44aa-9f48-1feaac3b9aac" }
            }
            function "PF3.2" {
                id: "79218945-69c6-4bc6-8aa6-9d67f537492d"
                parent: "f46f4ddc-51cc-4b2c-ba6c-d9e8c0bae8ea"
                port in "FIP 1" { id: "74312240-0e8d-411c-bc86-eb752dfb295a" }
                port out "FOP 1" { id: "4da12212-e97a-4916-8312-8c2a8f1a7ebe" }
            }
            function "PF4" {
                id: "0012d480-7be3-452f-9872-3a0d8868e2f6"
                parent: "f46f4ddc-51cc-4b2c-ba6c-d9e8c0bae8ea"
                port in "FIP 1" { id: "43db03cd-03cf-4229-b7ff-07cd5cee54a5" }
                port in "FIP 2" { id: "4d3bc0b0-7b1c-401e-ad14-f67bf48487f9" }
                port out "FOP 1" { id: "f167d6f3-2c18-4923-9ef2-cf774ccbff73" }
            }
            function "PF5" {
                id: "1291c3c4-170d-4fcc-9b59-5b0961cd4a2d"
                parent: "f46f4ddc-51cc-4b2c-ba6c-d9e8c0bae8ea"
                port in "FIP 1" { id: "368bf60e-8fff-4922-8bda-5ddff423bf95" }
                port out "FOP 1" { id: "12b2e0ec-c2ed-4ebf-b91d-ec793c6497cd" }
            }
            function "PF6" {
                id: "71147664-0fcd-401f-937a-bdc76216924d"
                parent: "f46f4ddc-51cc-4b2c-ba6c-d9e8c0bae8ea"
                port in "FIP 1" { id: "ae5a3c2f-cddf-49a9-8db3-2e127a300ea9" }
                port out "FOP 1" { id: "6fcece12-dc23-4a84-a6f7-b5d79aa1b938" }
            }
            function "PF7" {
                id: "62f152bd-6017-4828-9034-7af3c387f83a"
                parent: "f46f4ddc-51cc-4b2c-ba6c-d9e8c0bae8ea"
                port in "FIP 1" { id: "da6135bb-f373-40f4-ba5f-560201d7f735" }
                port out "FOP 1" { id: "1f3cb5dd-2b05-4f44-8267-ed246abfca57" }
            }
        }
        behavior_component "REC DEFINITION" {
            id: "eab59749-a0be-4ff3-9321-0d1e4d2f4451"
        }
        behavior_component "Distribution Network" {
            id: "84b3c2cb-d4ea-4cce-88e3-930c325463d4"
        }
        behavior_component "Unit Network Adapter" {
            id: "6e43a453-0966-4c19-93e5-c0a692c86941"
            function "Implement Network Configuration" {
                id: "e24662c4-47b2-445f-9d18-7214fca7013e"
                description: "The network distribution is an essential part of the IFE system. This function receives network configurations (IP address, maximum bandwidth, etc.) from the central network management and implements it on the component is it allocated to."
                parent: "71f18377-d79b-4619-a537-cfc07ecb8931"
                port in "FIP 1" { id: "f0c1cbdc-5fe6-44e9-a691-1d5b540a8e51" }
            }
            function "Run Network Tests" {
                id: "ccfb7dc0-c861-44db-95ea-0a216b9ec643"
                description: "This function replies to basic network tests."
                parent: "71f18377-d79b-4619-a537-cfc07ecb8931"
                port in "FIP 1" { id: "0ced1a83-7e58-4db2-87b9-223756fcfed6" }
            }
        }
    }
    node "IMPLEMENTATION PCs" {
        id: "5da29b86-88bd-4ced-9992-031690f87f99"
    }
    node "Video Server Unit" {
        id: "a850426c-c425-4c85-85fa-cd85b0c066e5"
        port "PP2" { id: "05870350-8156-4e09-b27b-b9cc86447e98" }
        port "PP 4" { id: "62d9e0f0-2b41-422a-aa33-02dd03925ed0" }
    }
    node "VSU Hard Disk" {
        id: "0cb2ede5-9ff4-417c-b6c4-c4e4f100ebed"
        port "PP1" { id: "003f9093-94e1-45d5-9f01-aa12569f52c5" }
        behavior_component "Media Server" {
            id: "eee15697-a375-4a9d-ba93-7ec5b34bc63c"
            port in "CP 1" { id: "424333b7-e718-447a-974f-9b399e82a98e" }
            port out "CP 2" { id: "020be0fd-2c42-4f0a-9b0b-7b4c229127f3" }
            port in "CP 3" { id: "585a7588-a300-456b-9158-dff00816e46d" }
            port out "CP 4" { id: "31fd76a9-47fa-4281-b8f8-92fb27ffb009" }
            port in "FIP 1" { id: "c7625346-50a0-4c57-b3f1-b2a284e709b9" }
            port out "FOP 1" { id: "49c7e84c-544c-4208-b244-e9f939ce3d92" }
            allocated_component: "_JNwHUJY7Ed6rH5SGn-UjSg"
            function "Store Digital Media" {
                id: "0378eba1-fb7b-4b4d-b39d-7a14ce4e5f4e"
                description: "This function is responsible for the storage of audio and video files that will be streamed and of movie information such as metadata (pictures,description)."
                realizes: "be9b5663-23ea-4413-abeb-3d5df84c93b8"
                parent: "54b31979-1ec1-49c5-81e0-512fd1aec704"
                port in "FIP 1" { id: "9fd7de66-1e58-49d2-bc23-2042b2ed8e0b" }
                port in "FIP 2" { id: "7c44c749-c66e-4fbe-b2ee-f18c03741219" }
                port in "FIP 3" { id: "5c2d7fc9-7b2d-4c0b-b9e6-39c321d8416a" }
                port in "FIP 4" { id: "3cbe9710-4013-434d-b004-9a9907adc06d" }
                port out "FOP 1" { id: "28f27523-cb5e-441c-8da2-92557e5e765f" }
                port out "FOP 2" { id: "ac255cd7-14c8-4289-a072-bd6b166f8e0d" }
                port out "FOP 3" { id: "b1c749d6-a0be-4290-91c1-9f455436e05c" }
                port out "FOP 4" { id: "d9fcde3a-7515-4ca1-b907-2f8fa3856094" }
                port out "FOP 5" { id: "46483a06-d71c-4b36-9295-6b1408f6ad0f" }
            }
        }
    }
    node "VSU Processor" {
        id: "c1cbb014-c2cd-403b-a9c1-62f438e79c8c"
        port "PP2" { id: "84970ec3-1fc3-4787-8ce6-90613a4b250b" }
        port "PP 2" { id: "487edb0c-f733-4d9a-a9eb-310f86c875d6" }
        behavior_component "Streaming Server" {
            id: "3f745bea-11ae-4b8c-8a06-930f2391565e"
            port out "CP 1" { id: "bbee94fd-1bf7-4709-bc8e-3de6dfe74908" }
            port out "FOP 5" { id: "77edb185-cc76-4fdb-84e8-d445bf8c871f" }
            port in "FIP 1" { id: "0a42c70e-a692-4e2a-8d89-1db4455b583c" }
            port in "FIP 2" { id: "3b9089e5-5836-4d43-923a-9770ea5ac793" }
            allocated_component: "_QHUAIJY7Ed6rH5SGn-UjSg"
            function "Initiate Audio-Video Stream" {
                id: "98960be5-3e8f-4743-ab70-805645472512"
                description: "This function is in charge of initializating a streaming operation. It reads the file(s) to be streamed in the digital media library and prepares the streaming header that is going to be sent to the target player (either seat TV or cabin screen). The stream header basically includes all information required to perform the decoding of the streams."
                realizes: "61bf6852-3f28-425d-b8ed-4d38b2b4eebb"
                parent: "cf0384ae-550f-40a4-a860-7d081b24a01e"
                port in "FIP 2" { id: "909fe395-29fd-4e41-93c3-ce4933f9fbe8" }
                port out "FOP 1" { id: "667cbf1f-5d7d-4ee7-8394-410f63d6627e" }
                port out "FOP 2" { id: "a597ef7f-19d4-4656-a1ae-dc6a90622261" }
            }
            function "Prepare Broadcasts" {
                id: "d53e6dd4-3926-4b25-80d1-14553a472f9e"
                description: "Thiis function is responsible for preparing broadcast material: Retrieving the digital content to be broadcasted from the storage unit and transmitting information about the target(s) of the broadcast."
                realizes: "27d03649-545b-40b0-8407-721165539005"
                parent: "cf0384ae-550f-40a4-a860-7d081b24a01e"
                port in "FIP 1" { id: "391350bf-5570-4833-81fd-437c5289e2de" }
                port in "FIP 3" { id: "29d6e9d1-8f9e-4421-8640-eaa28f937f7a" }
                port in "FIP 2" { id: "b0d84ad2-0697-46e1-b45b-8a9ad2914682" }
                port out "FOP 1" { id: "5888ab23-f353-4fea-b149-dc7b867940c4" }
                port out "FOP 5" { id: "24f7e363-1de6-41d0-9cfa-9eacabb62196" }
                port out "FOP 4" { id: "0f483ba5-b912-467d-85cb-c7d278519e50" }
                port out "FOP 2" { id: "72f5118e-f1a9-4699-a0a9-fdbc7fbf7fb5" }
            }
            function "Send Audio Video Packet" {
                id: "2dfb0a52-7568-4903-9c78-1e462848f868"
                description: "This function consists in transmitting audio and video packets to one or several Seat TVs and/or to the cabin screens."
                realizes: "61bf6852-3f28-425d-b8ed-4d38b2b4eebb"
                parent: "cf0384ae-550f-40a4-a860-7d081b24a01e"
                port in "FIP 1" { id: "701ca6d0-4009-4061-b3c1-4bfdafbdd9c0" }
                port out "FOP 1" { id: "a99e46e4-fc51-4c3c-9b87-89e04e130a62" }
                port out "FOP 2" { id: "66fd91f4-d4c3-4361-b7f3-41ad633f9049" }
                port out "FOP 3" { id: "6bd23657-2b1a-42f6-81e3-2e59bcdd15c6" }
            }
        }
        behavior_component "Unit Network Adapter - VSU" {
            id: "33ca7fc8-8bd1-48c3-9f22-3fbd217c2549"
            function "Implement Network Configuration - VSU" {
                id: "aecf56f9-9069-4d1d-87c5-3cd5fbd7e4fe"
                description: "The network distribution is an essential part of the IFE system. This function receives network configurations (IP address, maximum bandwidth, etc.) from the central network management and implements it on the component is it allocated to."
                parent: "03b0cf66-8fd1-4867-bec3-bc49814211a9"
                port in "FIP 1" { id: "dc58804e-1036-479f-bae6-5bdfc8b70c87" }
            }
            function "Run Network Tests - VSU" {
                id: "c4a29500-aeda-49ed-be51-65ca064266b6"
                description: "This function replies to basic network tests."
                parent: "03b0cf66-8fd1-4867-bec3-bc49814211a9"
                port in "FIP 1" { id: "a8560013-39c9-4c05-a4bb-f07d12618694" }
            }
        }
    }
    node "Ethernet Switches" {
        id: "ee7ecd3f-bf35-4657-b7bd-1e45a8fc025c"
        port "PP2" { id: "5aa6b85b-52c3-4070-9f7e-d68ffa00c50d" }
        port "PP 4" { id: "b383a439-0b94-4639-acdb-45ea7400a14b" }
        port "PP 5" { id: "4fe06916-33e1-4e46-9d90-1164bdf68972" }
        port "PP 7" { id: "87083a2b-9b2a-4246-b5ba-e2d46771628e" }
    }
    node "Private Video Display Unit" {
        id: "22cbea03-a250-4ea8-a6b3-63829196feb1"
        port "PP 1" { id: "23f6362b-85a2-4823-8ef9-131c26f8567f" }
        port "PP 2" { id: "75d6ebdd-5ca2-41f6-8c2e-a8320f3d874e" }
    }
    node "PVDU Processor" {
        id: "6d095cde-706f-41c2-91ab-7c5189f358c4"
        port "PP3" { id: "2477862f-0424-4c69-8a3a-118da3f61dc3" }
        port "PP 5" { id: "ca146e91-5744-4072-a8d7-ea290844ae9d" }
        port "PP 4" { id: "d0775105-2c33-4a40-a51e-c095397a776c" }
        port "PP 6" { id: "c65c2f62-d344-4df6-be3c-4aeb43dcf149" }
        behavior_component "PVDU Core Interactions Manager" {
            id: "647781a2-de41-4fa1-a6e6-3a886a706c4e"
            port in "CP 5" { id: "b347412a-12ea-4900-a8b2-44e01f9ff0ea" }
            port out "CP 4" { id: "96994cb6-5ec2-46c4-a307-db7d2e731cfc" }
            port out "FOP 2" { id: "6f4835f8-e8c2-44aa-a4fc-ae634f16a85a" }
            allocated_component: "_2SUXEJZFEd6rH5SGn-UjSg"
            function "Handle VOD Service Activation and Interruption" {
                id: "241dcfae-1880-45f9-bf2c-32b0cbe736b7"
                description: "This function is responsible for managing the VOD Service activations / deactivation (first activation, service resume, external interruption, etc.). This function relies on another function in charge of storing the current status of the interrupted / deactivated service (in particular, the current selection movie and the current position in this movie)."
                realizes: "910a61f1-3da0-404f-9db5-c226af9a3c47"
                parent: "f9353538-3fcb-4697-8591-daf861b7a164"
                port in "FIP 1" { id: "b10133d5-ef5b-4b4b-bedd-11e86b2c355d" }
                port in "FIP 2" { id: "5af16c6d-0031-4469-8078-55f2f3aa95c1" }
                port in "FIP 3" { id: "76ed6a23-8268-4ef1-b017-dcc8cfa978e2" }
                port out "FOP 1" { id: "fb42409e-7b38-4951-a21f-257448f35b5e" }
                port out "FOP 2" { id: "86c5e14e-7952-479a-9a97-4f1dcf899122" }
                port out "FOP 3" { id: "823bd8e2-ea94-464e-a625-d1db99bed28a" }
            }
            function "Process VOD Movie Controls" {
                id: "ac5be58c-aa92-4247-8c8f-9867ddbd6289"
                description: "This function transforms the VOD Movie play/pause/stop selections into applicative commands. The information about which Seat TV is requesting is attached to the created command."
                realizes: "faf3223f-3f47-4ad6-9c58-353b34e1a30f"
                parent: "8e79917f-4323-4040-b8c5-64c5e42c2a3b"
                port in "FIP 1" { id: "e9922603-6efb-438f-b559-c56a528b92f0" }
                port in "FIP 2" { id: "df2469d4-6751-4cb6-b616-f1421db07035" }
                port in "FIP 3" { id: "9d3fce16-996c-4c6b-8da8-21962808f6cb" }
                port out "FOP 2" { id: "bd32bf10-37a9-46a7-b40a-c8391429ee2b" }
            }
            function "Retrieve VOD Movie Data" {
                id: "dcbf11ee-150a-4afa-8e06-3f0ab9aee27a"
                description: "This function consists in receiving media information requests (list of VOD movies, available languages, etc.) and returning the data corresponding to these requests. This data is retrieved from the digital media storage unit."
                realizes: "651179c3-78b6-4a6f-a174-74405b81b36a"
                parent: "f9353538-3fcb-4697-8591-daf861b7a164"
                port in "FIP 1" { id: "77588645-feb2-4c98-a043-d45761432acf" }
                port in "FIP 2" { id: "8884252e-1db7-4419-9150-6a544f8e12b6" }
                port in "FIP 3" { id: "fbeb1e96-128e-49e9-8b3b-6c0724424313" }
                port in "FIP 4" { id: "adf4e530-5c7c-4dba-95f7-731e086e1564" }
                port in "FIP 5" { id: "c2ce5c06-fa05-4432-b4e2-dccce61ed9d3" }
                port in "FIP 6" { id: "6eb9c221-e2a6-4089-9732-9ac7845d1e7c" }
                port in "FIP 7" { id: "c3d698ee-fa2e-46a2-86ed-6093c4f0bb08" }
                port in "FIP 8" { id: "c24a2186-a2c9-4a7e-9c0e-424e38e93c14" }
                port out "FOP 1" { id: "5d0ba8aa-17dd-4bd0-bf08-e6b350165c15" }
                port out "FOP 2" { id: "ed52e281-4b73-458a-a754-09ffedafae95" }
                port out "FOP 3" { id: "01922166-eca4-4552-b7bf-2ce5849146e5" }
                port out "FOP 4" { id: "ad41e710-3a13-40ca-b04d-8d877453d1ce" }
                port out "FOP 5" { id: "c79c3f6e-5a4b-47ea-ac7a-0168539ae409" }
            }
            function "Start/Resume Service" {
                id: "8cc1e6f5-dbe2-4629-8a43-b60eef771a8f"
                description: "This function displays the available services and trigger service activations when applicable."
                realizes: "c3b17f24-ae52-413f-ace7-6baab1287b02"
                parent: "2e06d355-e036-40f8-9f84-074d413e7b07"
                port in "FIP1" { id: "0209385f-1963-41bd-94f6-0fab86a65a91" }
                port in "FIP 2" { id: "7ee867c0-a56a-42c9-aa8d-06ef91add4af" }
                port in "FIP 4" { id: "5b8f1b5a-8b83-4b39-8b76-5d634668ae42" }
                port out "FOP 1" { id: "bc0ad6e9-a5ab-41eb-a880-2ea4635df713" }
                port out "FOP 4" { id: "05762aae-3b7e-4b6b-912d-02997976dbaf" }
            }
        }
        behavior_component "PTV Video Player" {
            id: "819adf3f-da1d-4d2b-8faf-3b1d7b99503d"
            port in "CP 1" { id: "a6da27c4-3ca9-4790-8026-08b66e848ec0" }
            port out "CP 2" { id: "5c8f6d97-2879-48d9-a1c9-04584f93e1eb" }
            function "Split Audio Video Streams" {
                id: "4c361e00-d6d0-4acf-a523-1de11acf7797"
                description: "Encoded audio and video streams are typically assembled in a container bitstream such as MP4 or FLV. This function is in charge of splitting the two streams that are going to be decoded separately."
                parent: "3d159790-b7eb-46d7-90ac-d64017225cf0"
                port in "FIP 1" { id: "218682ca-b3b1-49ba-946f-cbf59a169751" }
                port out "FOP 1" { id: "8eb0fb9c-64e4-494e-a0f3-eaa739ccb52e" }
                port out "FOP 2" { id: "f1ab914e-8ef1-4be7-a94e-7092f7ed29fa" }
            }
            function "Store interrupted VOD Movie Status" {
                id: "8daae3ab-944b-4ade-9896-0d51c62ef3e8"
                description: "This function is in charge of storing the current status of the interrupted / deactivated VOD service (in particular, the current selection movie and the current position in this movie)."
                realizes: "3136218c-bf14-437d-99c9-6593352eead9"
                parent: "f9353538-3fcb-4697-8591-daf861b7a164"
                port in "FIP 1" { id: "6d0100a3-1646-4fe9-8ee7-43f107384f5f" }
                port out "FOP 3" { id: "648b872c-1bdd-4b2d-acd0-379fce38ce86" }
            }
            function "Synchronize Audio Video" {
                id: "1289a361-a3e8-416b-a9e0-e4fcd528d9e3"
                description: "This function is responsible for providing the decoding functions with the synchronization information needed to play back the audio and video streams together. In most cases, the file header, most of the metadata and the synchro chunks are specified by the container format."
                parent: "3d159790-b7eb-46d7-90ac-d64017225cf0"
                port in "FIP 2" { id: "a490ab74-0d8f-44f5-9ce4-7a3874235e16" }
                port out "FOP 1" { id: "431e58cb-8de2-4055-af5c-286817c5c2b5" }
                port out "FOP 2" { id: "bd57e760-35e5-4c58-adf5-0049f2e2789e" }
            }
        }
        behavior_component "PVDU Services Manager" {
            id: "737331c9-23e8-4588-969a-fd15444cafa6"
            allocated_component: "e02be1c5-a1a2-4c11-be30-259e92ef5826"
            function "Interrupt Current Service" {
                id: "3f69f8bd-704d-438b-8f05-fc7d56363811"
                description: "This function is responsible for triggering passenger service interruptions as soon as an imposed video is played or an audio announcement is performed. When a VOD movie is running, this means for example that the movie as to be automatically paused and the current position stored."
                realizes: "b4623d17-6e62-411d-839c-2d768664a920"
                parent: "cf7dca25-2250-4339-9501-eaf3557a644c"
                port in "FIP 3" { id: "fd9b6cbe-a150-4eca-8674-8f0237265097" }
                port in "FIP 2" { id: "7828e3d7-d29f-4147-a02a-1fac8e5f86be" }
                port out "FOP1" { id: "406befbd-573e-479e-b839-035a5d87c2ef" }
                port out "FOP 2" { id: "03448010-aee9-46d6-a4bb-8bdaeb269df5" }
            }
            function "Store Passenger Service State" {
                id: "8835c15a-0eb0-4d20-83d2-a15f8c0bd005"
                description: "Whenever a service is interrupted, its current state has to be saved, in order to be able to resume it later on in the same state. This function typically delegates this action to the functions directly responsible for running each passenger service."
                realizes: "3e9ccc30-596e-40e6-8941-64838248dcee"
                parent: "2e06d355-e036-40f8-9f84-074d413e7b07"
                port in "FIP 1" { id: "6750ef71-9cdf-4d69-9a7a-caf60065a04b" }
                port in "FIP 2" { id: "df58bdad-683c-4f8d-bc7e-5df3bd5ca7d9" }
                port out "FOP 3" { id: "6bdf9f28-6ae4-4cc1-a681-9137880f1b7a" }
            }
        }
        behavior_component "Unit Network Adapter - PVDU" {
            id: "809ec42b-c1ee-471f-a45b-d26977dc956a"
            function "Implement Network Configuration - PVDU" {
                id: "84c8b355-ee9b-48dd-aca8-17b0c4e477c8"
                description: "The network distribution is an essential part of the IFE system. This function receives network configurations (IP address, maximum bandwidth, etc.) from the central network management and implements it on the PVDU."
                parent: "03b0cf66-8fd1-4867-bec3-bc49814211a9"
                port in "FIP 1" { id: "cdf5f18d-9a3b-43c4-9ba7-d1fb5eb01e0d" }
            }
            function "Run Network Tests - PVDU" {
                id: "b2fe1c12-86f0-4c40-9986-d349c2a09496"
                description: "This function replies to basic network tests."
                parent: "03b0cf66-8fd1-4867-bec3-bc49814211a9"
                port in "FIP 1" { id: "765f7870-ec06-4de1-9c46-a9633fbbb52a" }
            }
        }
    }
    node "PVDU MPEG Decoder" {
        id: "3d96ae05-f435-49b3-abd1-574551f38704"
        port "PP 2" { id: "56cfcaa4-2655-4ea9-ba84-9f867c82116b" }
        behavior_component "PVDU Video Decoder" {
            id: "34420358-2236-4010-8504-5fab9af55c0d"
            port in "CP 1" { id: "6912b6da-a7f6-4881-999e-a31249583b32" }
            function "Decode Video Packets" {
                id: "9d3a7965-d967-42e2-8f25-c0113abaf9ff"
                description: "This function consists in decoding the video packets typically compressed using a video codec such as H264.This function receives metadata allowing the synchronization with the audio decoding performed separately."
                parent: "d8e4de58-aebd-43de-85d6-b7e0011cf619"
                port in "FIP 1" { id: "0ae12d46-c4e2-47f1-a894-ecb7c6e15206" }
                port in "FIP 2" { id: "197398b6-59ff-42a0-998b-0b9ef44987e3" }
                port out "FOP 1" { id: "e76ea700-5b88-4258-bdf6-7a93e9c6a2f0" }
            }
            function "Receive and Bufferize Video Packets" {
                id: "ecc7d466-f9cb-4be1-a099-b8449146cf18"
                description: "As the video data packets arrive, they are buffered for a few seconds and then decoded for the playback to begin. Bufferizing the video packets reduces the risk of playback interruption due to network congestion."
                parent: "d8e4de58-aebd-43de-85d6-b7e0011cf619"
                port in "FIP 1" { id: "32c90801-f18f-4067-a354-0bb22f3ae0fc" }
                port in "FIP 2" { id: "b7ac512d-2708-4140-9d4c-184a8ade1e30" }
                port out "FOP 1" { id: "d2ea59a5-33c9-4a19-bd1f-81721fe9840e" }
            }
        }
    }
    node "PVDU Screen" {
        id: "44003100-30d6-4894-af55-e2e982a5b921"
        port "PP2" { id: "83d7c877-328c-4224-adf0-b69c1a043b5b" }
        behavior_component "PVDU Screen SW" {
            id: "d187e156-9b25-4adc-8b97-576853d16e40"
            function "Display Audio Interruption Screen on Seat TV" {
                id: "068dc2db-3e29-421d-84be-8444ce8acc9c"
                description: "When an audio announcement is made, all IFE services on Seat TVs are interrupted. A message is then displayed on the screen, indicating that an audio annoucement is currently ongoing."
                realizes: "4ff700ef-4f6b-4151-917b-e59064e5a225"
                parent: "3d159790-b7eb-46d7-90ac-d64017225cf0"
                port in "FIP 1" { id: "4c4032c5-67b2-4cf0-b3c0-4a73c1974766" }
                port out "FOP 1" { id: "f17d1840-57bf-47ca-86e0-e3a2fdeddc75" }
            }
            function "Display Homepage on Seat TV" {
                id: "83b4863a-7795-4911-aeef-2938443cce3d"
                description: "This function simply displays the Seat TV starting page, from where all services can be launched."
                realizes: "70f1acf3-a32d-4190-b97e-3eb361a559e7"
                parent: "cf7dca25-2250-4339-9501-eaf3557a644c"
                port in "FIP 1" { id: "70e193ad-32f0-49e4-949b-45053eba5c8f" }
                port in "FIP 2" { id: "a8617529-d2ae-4ec4-8cd2-eb97f54f09b9" }
                port out "FOP1" { id: "68ef39ea-ae2e-4660-aa21-5febcff5976c" }
                port out "FOP 2" { id: "54caca62-7ec3-4764-a64a-5aaab9747250" }
            }
            function "Display VOD Movie Data" {
                id: "c7edaab3-11bc-4409-b300-13c4b4192742"
                description: "This function is responsible for displaying all the VOD service information on the Seat TV."
                realizes: "94aebe8b-af13-4a7a-a7ee-8fd10a7fe489"
                parent: "f9353538-3fcb-4697-8591-daf861b7a164"
                port in "FIP 2" { id: "c3fe51bf-3616-4fd3-af0f-afa536cb6360" }
                port in "FIP 1" { id: "61e5363b-9b25-4636-aa0f-81aa890dd7f8" }
                port in "FIP 3" { id: "c66261d3-11e2-49e8-9440-83435c61c7dc" }
                port in "FIP 4" { id: "3ab1eb98-2f61-417b-be5f-c31fef1abbc6" }
                port out "FOP 1" { id: "d8457f55-d830-43fe-9222-0c60b6a2435c" }
                port out "FOP 2" { id: "69eab9cf-d746-405f-be1c-72eb85bf8a10" }
                port out "FOP 3" { id: "49f27eb4-a68d-4a56-872a-698b8efd280a" }
            }
            function "Display Video on Seat TV" {
                id: "bd383ff2-258a-41dd-973f-07f7aa30da7a"
                description: "This function is in charge of displaying video frames on the seat TV."
                parent: "3d159790-b7eb-46d7-90ac-d64017225cf0"
                port in "FIP 1" { id: "7f56627e-14f4-45fb-b6c4-31102daba398" }
                port out "FOP 1" { id: "977cc5d0-937a-4907-acb9-890d199d071d" }
                port out "FOP 2" { id: "ce410bbc-b015-47f7-9228-7ce6dd9460a7" }
            }
        }
    }
    node "PVDU Audio Processing Unit" {
        id: "78c650a1-7440-4335-bf95-f421abdf2c0f"
        port "PP 1" { id: "aaf6597a-2a5a-4f05-9c01-0a9f0d368715" }
        behavior_component "PVDU Audio Decoder" {
            id: "3de7180a-c2f0-48ea-9a9e-6f6e6849c651"
            function "Decode Audio Packets" {
                id: "07823fa4-38c9-43fe-8b4b-8aa5588b7c46"
                description: "This function consists in decoding the audio packets typically compressed using an audio codec such as MP3 or AAC.This function receives metadata allowing the synchronization with the video decoding performed separately."
                parent: "6f47362b-60ef-451f-8a00-005ebd4fd010"
                port in "FIP 1" { id: "c64da8d7-6985-4d1d-8c04-624dfb385634" }
                port in "FIP 2" { id: "d6abeae3-1f3a-46cf-b1be-d3228e01e0f5" }
                port out "FOP 1" { id: "7937467b-0cea-4eee-ae99-bf47dee40f04" }
            }
            function "Receive and Bufferize Audio Packets" {
                id: "9155e7ae-b4c3-4f06-b8b2-0ec90045ac8e"
                description: "As the data packets arrive, they are buffered for a few seconds and then decoded for the playback to begin. Bufferizing the audio packets reduces the risk of playback interruption due to network congestion."
                parent: "6f47362b-60ef-451f-8a00-005ebd4fd010"
                port in "FIP 1" { id: "3daead22-5959-46c8-b97d-553bfa048cdd" }
                port in "FIP 2" { id: "5e3392bd-1086-43f4-b9a1-9c47f61fbad9" }
                port out "FOP 1" { id: "834c7563-4422-4eff-aba8-4de93ee6bfb9" }
            }
        }
    }
    node "BUS" {
        id: "2632079f-6a54-406a-a957-011c53b16209"
        port "PP 1" { id: "dc092d53-ac0e-4710-9f34-8cd204371a9e" }
        port "PP 2" { id: "9443a277-bb07-491d-a5ab-4c2b53423c65" }
        port "PP 3" { id: "6bf4f507-74d4-41a5-8f69-701e6f87b675" }
        port "PP 4" { id: "da8c6820-127e-4082-9bb2-b84c54161a74" }
    }
    node "Digital Analog Converter" {
        id: "e93cee23-c12e-43e4-ac6d-6049aeb13a63"
        port "PP 1" { id: "7359adad-2eb3-4030-b9f6-d24aaa1c379c" }
        port "Jack" { id: "75480df0-307e-4265-b770-6bb574148d94" }
        behavior_component "DAC SW" {
            id: "3cba132d-d6ec-4839-b597-2183f9f998b2"
            function "Convert Audio" {
                id: "a84dff3b-2158-41ed-b96c-0b8ace0bea3b"
                description: "This function converts a digital audio stream in a analog one."
                parent: "3d159790-b7eb-46d7-90ac-d64017225cf0"
                port in "FIP 2" { id: "38d41706-b9a0-4d6b-91a7-3dedb565a7c7" }
                port out "FOP 1" { id: "fac41245-a395-47f8-ae16-97ccec9ef9e9" }
            }
        }
    }
    node "Passenger Remote Control" {
        id: "5ed18efd-1535-4aa3-aaaf-48a3ca867607"
        port "PP1" { id: "6626e901-0576-4ecb-9163-a03f3c7303d2" }
        behavior_component "Remote Control SW" {
            id: "ec105112-c640-4de6-bc28-461581a1bace"
            function "Capture Passenger Service Selection" {
                id: "9d76fefa-2f89-4272-8b1d-785d05601ce6"
                description: "This function&#160;handles the general interactions with the passenger. The Seat TV displays&#160;interaction widgets (typically buttons&#160;and slide bars). This typically airline-specifc, because user interface typically is. This function is responsible&#160;for&#160;capturing the selections of&#160;services and forwarding them to&#160;a more generic function&#160;(a controler) that will transform the selections into an internal system commands (activation / de-activation).&#160;&#160;"
                realizes: "2ea2703a-6dd1-4835-aaae-16179864ac1d"
                parent: "cf7dca25-2250-4339-9501-eaf3557a644c"
                port in "FIP 1" { id: "9390455f-c29b-490f-8628-b51c2c901faa" }
                port in "FIP1" { id: "2f162340-ca52-4838-82e9-5232866766ac" }
                port out "FOP 2" { id: "03a839ec-5719-4397-a3c7-905daed36739" }
                port out "FOP1" { id: "9c17c4e4-c33b-4726-a705-5b26f7541e10" }
                port out "FOP 1" { id: "b59eefe5-797d-41c4-893b-aa75aefc96a1" }
            }
            function "Capture VOD Selections" {
                id: "0c8009a9-0524-4445-a9c8-f0295467f62a"
                description: "This function&#160;handles the interactions with the passenger in the VOD service. The Seat TV displays&#160;interaction widgets (typically buttons&#160;and slide bars). This typically airline-specifc, because user interface typically is. This function is responsible&#160;for&#160;capturing these selections and forwarding them to&#160;a more generic function&#160;(a controler) that will transform the selection into an internal system command. &#160;"
                realizes: "5165a225-e662-4b4a-95cd-c94fc5726524"
                parent: "f9353538-3fcb-4697-8591-daf861b7a164"
                port in "FIP 1" { id: "4c653d71-de95-4a67-963f-69ed77895bed" }
                port in "FIP 2" { id: "e9eab652-737b-4d13-8960-0fb1c4a43715" }
                port in "FIP 3" { id: "88aec256-9cbb-415d-8f43-278f059c85cb" }
                port in "FIP 4" { id: "29d17e74-827f-4ff0-9ce8-2133378c7887" }
                port in "FIP 5" { id: "0944dc86-fd5b-42ed-aba2-0c7ee0ce5f99" }
                port out "FOP 1" { id: "409bd923-08c7-4757-92d4-71d2306d72a0" }
                port out "FOP 2" { id: "4d97a6e9-9d65-45c6-8001-dce52df44a4d" }
                port out "FOP 3" { id: "b15ebc2b-b166-4022-bc15-14f6066f8b42" }
                port out "FOP 4" { id: "5b9082ec-31b5-4072-a30b-3398d3d63793" }
                port out "FOP 5" { id: "f2b8c0df-14e6-414f-8e30-6babdefd5147" }
                port out "FOP 6" { id: "f080e12d-abaf-47c0-924c-d77fa6fa7b3f" }
                port out "FOP 7" { id: "46ba1204-0992-4ffc-9739-4a6a50fc6622" }
            }
        }
    }
    node "Cabin Management Unit" {
        id: "01cf8dac-9d35-43aa-a717-5e740989fdc1"
        port "PP 4" { id: "b3c52875-96c4-4dba-bfac-cbc62528bb78" }
    }
    node "CMU Processor" {
        id: "fec9b0d7-7a0b-4386-b1e3-4cccb2c721c3"
        port "PP 1" { id: "7207de1d-632e-4e54-b364-8dc093af23ee" }
        port "PP 2" { id: "ded1883e-416b-4e58-8eae-170d8bf13bb4" }
        behavior_component "CMU Core Interactions Manager" {
            id: "a6815c30-8c2b-4273-bbf1-450bb1df58ed"
            port in "CP 5" { id: "6a13f6bb-e86e-4926-8527-5a65e10ba38d" }
            port out "CP 4" { id: "916bbd6b-2cdb-46eb-8b2e-f66bcdf05eb0" }
            allocated_component: "01a67cde-a09c-446e-9637-ab235664d23c"
            function "Process Imposed Video Controls" {
                id: "737aebcf-1f00-40ac-8e03-df8bc6596159"
                description: "This function transforms the Imposed Video play/pause/stop selections into applicative commands."
                realizes: "7069fd31-6eb0-49fa-ae52-0b78307c2748"
                parent: "8e79917f-4323-4040-b8c5-64c5e42c2a3b"
                port in "FIP 2" { id: "eba05709-f182-41ed-801a-fe0e173bdb4b" }
                port in "FIP 4" { id: "aaf69f5c-880b-4300-bbca-0800242ccac7" }
                port in "FIP 3" { id: "12022972-662f-4386-bfd7-b4708160a8a0" }
                port out "FOP 3" { id: "a614a705-30d7-4355-bc38-47d9136fa94b" }
                port out "FOP 4" { id: "6b74285f-81a5-4213-a86b-66aae4fe8bbd" }
            }
            function "Receive Imposed Video Playing Status" {
                id: "e8af73d3-bacb-4222-b387-f6b0bd4d13a1"
                description: "This function collects the status of the currently running imposed movie (ETA, etc.). This status is then transmitted to be displayed on the Cabin Management Terminal."
                realizes: "b55db2ae-a407-4c31-8093-75f0841094eb"
                parent: "3d159790-b7eb-46d7-90ac-d64017225cf0"
                port in "FIP 4" { id: "ba306852-f3c1-40d5-8164-e7e216497397" }
                port in "FIP 1" { id: "01060b20-93f7-47c4-b147-76f0179c1e90" }
                port out "FOP 1" { id: "24479f69-5171-40af-84c5-0521a6ba332e" }
            }
            function "Retrieve Imposed Video Data" {
                id: "1cda3b05-ee17-4912-b875-40bc4ddca86b"
                description: "This function consists in receiving media information requests (list of imposed videos, available languages, etc.) and returning the data corresponding to these requests. This data is retrieved from the digital media storage unit."
                realizes: "0d727549-f01f-4cc6-ae27-5f25220ece06"
                parent: "46a33304-1338-4fc0-a9cb-dc3fe19e5ed4"
                port in "FIP 1" { id: "88f5f9ef-6670-4495-8fa0-85294027c5d1" }
                port in "FIP 2" { id: "78869c33-b002-42da-a431-2f43b18d472f" }
                port in "FIP 3" { id: "1287de60-3ceb-4ad3-8ece-5306406bb769" }
                port out "FOP2" { id: "2397abae-882b-4f4d-9d39-b255f2687620" }
                port out "FOP 2" { id: "b8ce9590-6f57-4778-9dc9-9f95766ce9e1" }
            }
        }
        behavior_component "Unit Network Adapter - CMU" {
            id: "403273f6-e4ae-4eed-90f3-d566ecffb01a"
            function "Implement Network Configuration - CMU" {
                id: "c5503a00-fae8-439d-90a7-6b8644d2f609"
                description: "The network distribution is an essential part of the IFE system. This function receives network configurations (IP address, maximum bandwidth, etc.) from the central network management and implements it on the CMU."
                parent: "03b0cf66-8fd1-4867-bec3-bc49814211a9"
                port in "FIP 1" { id: "cf123dcb-5d86-47fa-9dbb-e6670495bfdb" }
            }
            function "Run Network Tests - CMU" {
                id: "238797f1-c012-446a-9b79-e94cf88bd180"
                description: "This function replies to basic network tests."
                parent: "03b0cf66-8fd1-4867-bec3-bc49814211a9"
                port in "FIP 1" { id: "78736f13-8b41-4119-b0b4-48e733598c85" }
            }
        }
    }
    node "CMU Touch Screen" {
        id: "f52035f8-64d3-4259-8aef-5ad19157efca"
        port "PP 1" { id: "42bbf8c7-aeaa-4ca8-b21f-8e15de4b18dd" }
        behavior_component "CMU Airline-Specific Interactions Manager" {
            id: "abc69d28-d5c4-470e-afd1-89daa1cc81d2"
            port in "CP 2" { id: "599059c6-d014-4264-a60c-915261473508" }
            port out "CP 1" { id: "5457ca00-0304-4796-8b56-b6f0c79b0526" }
            allocated_component: "97e2a56c-0d4c-4a56-924a-63a31e6c50d7"
            function "Capture Imposed Video Selections" {
                id: "9b798c7d-2d5a-43f9-867d-b26c3f4c1337"
                description: "This function&#160;handles the interactions with the&#160;cabin crew on the cabin management terminal.&#160;The terminal&#160;displays&#160;interaction widgets (typically buttons&#160;and slide bars). This typically airline-specifc, because user interface typically is. This function is responsible&#160;for&#160;capturing these selections and forwarding them to&#160;a more generic function&#160;(a controler) that will transform the selection into an internal system command."
                realizes: "1aa42ce5-1c2c-4473-9840-7d20a60c377e"
                parent: "46a33304-1338-4fc0-a9cb-dc3fe19e5ed4"
                port in "FIP 1" { id: "b18f21aa-5e65-4740-8f22-ac0661fdb02c" }
                port in "FIP 2" { id: "6d6577f7-4ae4-49e8-9c07-aab7af5c6505" }
                port in "FIP 3" { id: "067db067-addf-4764-88f7-d5a7aa41e09c" }
                port in "FIP 4" { id: "7e5b7b63-c670-40c9-b655-c2afdce91c9c" }
                port out "FOP 2" { id: "234fd1b3-e8fb-40c4-a9c7-373aa913565f" }
                port out "FOP 3" { id: "6c9b3f09-ce19-4598-a98c-62c589553975" }
                port out "FOP 4" { id: "8c4d39e7-2bfe-49be-8b15-13fe66adb74b" }
                port out "FOP 5" { id: "818dd0f9-e8ff-4d4b-8737-92c5ca41c5c6" }
            }
            function "Display Imposed Video Data" {
                id: "cebc5976-81d6-4718-b1f0-9f8b7687ceeb"
                description: "This function is responsible for displaying all the imposed video data on the cabin management terminal."
                realizes: "a90e98e8-741e-478a-9e14-af7a50ee2525"
                parent: "46a33304-1338-4fc0-a9cb-dc3fe19e5ed4"
                port in "FIP1" { id: "66871b36-e273-4b9b-9fb7-f86ab00be65e" }
                port out "FOP1" { id: "1f31f671-ff1f-4a9c-ade6-95bc9f5ab26a" }
                port out "FOP2" { id: "39181415-5117-4484-8b3d-9d2502d76692" }
            }
            function "Display Imposed Video Playing Status on Cabin Terminal" {
                id: "72f0a4da-866e-4136-8ca7-bfc85db05005"
                description: "This function is responsible for displaying the status of the currently running imposed movie (ETA, etc.) on the Cabin Management Terminal."
                realizes: "566f8900-7408-42d3-82f8-ba18bf0d923f"
                parent: "3d159790-b7eb-46d7-90ac-d64017225cf0"
                port in "FIP 1" { id: "3905e1b7-3687-48d5-a13e-4f550c01ae49" }
                port out "FOP 1" { id: "b401dd9c-a0dc-4cdb-a48b-8c8d1ef160f5" }
            }
        }
    }
    node "Media Loading Unit" {
        id: "5f666fd8-8032-45f7-821f-1bccbe117b50"
        port "PP1" { id: "c643c0e6-0287-4448-8911-b45dc113dd3e" }
        behavior_component "Media Loading Unit SW" {
            id: "96cfbcc8-37da-4181-b53f-155f8160db5d"
            function "Load Digital Media" {
                id: "16904687-4018-46cc-a0ec-c415f6d2bc1d"
                description: "This function is responsible for receiving the media content uploaded by the ground operator and for moving it onto the IFE media storage unit."
                realizes: "70d62f70-d0b6-491c-9e9e-f5ed77179ce4"
                parent: "54b31979-1ec1-49c5-81e0-512fd1aec704"
                port in "FIP 1" { id: "294efc47-a2ee-4360-af86-5997b4b15118" }
                port out "FOP 1" { id: "feccacfa-08ae-4427-a36c-c7807b98db03" }
            }
        }
    }
    node "Application Server HW / SW" {
        id: "3e6b5b54-0616-4c9e-a017-86f574e2e98f"
        port "PP 2" { id: "6631f440-3a9b-48ba-8e5d-f0f70dcbeaa5" }
        port "PP 3" { id: "b14a3b42-301b-4727-9a2f-76f99394d093" }
    }
    node "ASU Processor" {
        id: "820e9f56-cac0-4043-ba05-19763e6e3788"
        port "PP 1" { id: "bf993152-af6f-4e99-89c8-13b822e3e8e9" }
        port "PP 2" { id: "9c259046-60dc-450a-833d-359802635f6c" }
        behavior_component "Aircraft-Specific Aircraft Interface" {
            id: "cdc16f53-6eaa-4d74-8d63-73afc483dece"
            allocated_component: "_SF1a8JY-Ed6rH5SGn-UjSg"
            function "Acquire Audio Stream from Aircraft" {
                id: "2788898f-6daf-43c1-bca8-6ab484ac24c6"
                description: "Audio announcements are performed through the aricraft built-in sound system (microphone, speakers, etc.). The IFE system is responsible for dispatching these announcements to each Seat TV. This function receives the audio streams from the aircraft and forwards it to the IFE applicative server. The implementation of this function is likely to be specific to each aircraft."
                realizes: "ee920873-7e2d-487f-a51e-76236d6b094e"
                parent: "1da11685-85e8-4ea3-a6e8-e7cfe33111d4"
                port in "FIP 1" { id: "3f0466f8-8122-44b0-87b8-61c195bde66d" }
                port out "FOP 1" { id: "ebd60392-0235-4ce1-ac3c-795b82f6506b" }
            }
            function "Capture Aircraft Notifications and Parameters" {
                id: "b4c36894-10af-4166-9da7-099de70ad0f6"
                description: "This function is concentrating the reception of the messages / notifications sent by the aircraft. The implementation of this function is likely to be specific to each aircraft model: The format of the incoming messages is defined by the aircraft manufacturer."
                realizes: "4b01ac5e-7920-491d-af35-1846163e5ad9"
                parent: "1da11685-85e8-4ea3-a6e8-e7cfe33111d4"
                port in "FIP 1" { id: "29281d72-e1d4-4f26-b78d-1c657a36fab8" }
                port in "FIP 2" { id: "fc9728ae-ab7b-4ff9-949d-63b082c60c25" }
                port out "FOP 1" { id: "371ad168-c5c7-43f7-bcd3-6d0331a3aa56" }
                port out "FOP 2" { id: "fe83f69e-3b1d-4c26-ae4b-3d46ce069dc8" }
            }
            function "Send Audio Stream to Aircraft Cabin Speakers" {
                id: "ccf41abe-2a9d-4f50-bc3d-643397b7a338"
                description: "This function receives the audio track of imposed videos and forward them to the aircraft. The video part is played on the Cabin Screen through another dedicated function."
                status: "REWORK NECESSARY"
                review: "Modéliser le décodage"
                realizes: "30d03eaa-2615-481c-bf1f-53f177d8147c"
                parent: "3d159790-b7eb-46d7-90ac-d64017225cf0"
                port in "FIP 2" { id: "1dba4c09-f51b-47db-9e69-a2e8e2c8639d" }
                port out "FOP 1" { id: "cb346dbd-5582-47a9-938e-fe65c2d921b6" }
            }
        }
        behavior_component "Core Aircraft Interface" {
            id: "f559c928-27a8-4198-9b0e-fa26404785cd"
            allocated_component: "_Rml6MJY-Ed6rH5SGn-UjSg"
            function "Determine Operating Profiles" {
                id: "c13d6970-0ca4-44b4-be4b-64b6f3eebbb4"
                description: "Based on the notifications it receives from the function interfacing with the aircraft, this function computes the mode in which the IFE system has to operate. For example, if the aircraft sends a power reduction request, the IFE may switch to a downgraded mode."
                realizes: "1e812d3d-6c35-42c9-80fa-eb64dfc1a3a8"
                parent: "c75359fb-2271-4499-8199-c3e1565697a7"
                port in "FIP 1" { id: "4b9fcc04-dca7-4d37-aff2-8eb38bc4f027" }
                port in "FIP 2" { id: "f69d694d-8984-4459-91a0-528d20d821b1" }
                port out "FOP 1" { id: "5cfd52bf-4fe5-4abc-a63c-d09e03b04992" }
            }
        }
        behavior_component "Aircraft-Specific Network Manager" {
            id: "af29b99b-0372-4534-906d-6038ff266deb"
            function "Define Initial Network Configuration" {
                id: "3fcaa02a-0321-4be3-ab79-b2547d47749b"
                description: "The network distribution is an essential part of the IFE system. This function creates the configurations for all connected equipments (IP address, maximum bandwidth, etc.)."
                parent: "7f4ecc96-3b56-4470-a35b-ce764d0b4e1f"
                port out "FOP 1" { id: "f189007a-281e-4466-839f-2fd3ea91be1a" }
            }
            function "Orchestrate Network Tests" {
                id: "e323d7b2-2bb2-4ffe-a1b5-265c98befe28"
                description: "This function is responsible for triggering networking tests on connected equipments."
                parent: "7f4ecc96-3b56-4470-a35b-ce764d0b4e1f"
                port out "FOP 1" { id: "42376149-712d-40da-bf42-6866cc21efbb" }
            }
            function "Reconfigure Network" {
                id: "6aa6df97-e744-4b1b-962f-66874bfabbaa"
                description: "The network distribution is an essential part of the IFE system. This function updates the configurations for connected equipments (IP address, maximum bandwidth, etc.)."
                parent: "7f4ecc96-3b56-4470-a35b-ce764d0b4e1f"
                port out "FOP 1" { id: "b66ca5bd-d104-4b16-b258-934b108edf6f" }
            }
        }
        behavior_component "Applications Server" {
            id: "a2bf21fb-902c-4b29-87c3-d3436dedb1db"
            allocated_component: "df043998-7fd1-4d4d-899f-7257788d644f"
            function "Determine Passenger Service Availability" {
                id: "88a4d6d0-fcf2-4c03-be5e-e6a23c727bf4"
                description: "This function is responsible for computing the availability of passenger services based on the authorization scheme (airline business strategy) and the aircraft contraints."
                realizes: "5a58fd72-1b57-41dc-8b7b-b0b8cf577236"
                parent: "cf7dca25-2250-4339-9501-eaf3557a644c"
                port in "FIP 1" { id: "4b0792b2-5e89-4beb-873c-c76df4584607" }
                port in "FIP 2" { id: "55e2e072-e14e-411c-b692-1d6946a1105d" }
                port in "FIP 3" { id: "5786cf01-9321-49aa-8f25-022d1c600e33" }
                port out "FOP 1" { id: "a36f1507-4b64-4293-8b43-c4e76177bcce" }
            }
            function "Process Audio Announcement" {
                id: "80a8d0f6-ca9e-486f-9ac3-0c524de61281"
                description: "When an audio announcement is made (thoufh the aircraft equipment), the IFE system is responsible for forwarding the audio stream to all passengers. This function also triggers a passenger service interruption command"
                realizes: "39f96e89-8ead-4742-bc1f-6eb0dd33990b"
                parent: "cb0426b2-9cab-4a4d-9e7b-89a13a57be2e"
                port in "FIP 1" { id: "a290721c-ce19-4899-8070-2df9a1dd4647" }
                port out "FOP 3" { id: "5f6cd196-8a51-49f5-b7ca-e13c352eb6f9" }
                port out "FOP 2" { id: "2c45a56b-91fa-43b7-994a-0cb89254dab3" }
            }
        }
        behavior_component "Cabin Video Player" {
            id: "ce7ff26b-6729-420e-b6a7-78fb80634d88"
            function "Decode Video Packet" {
                id: "c91204f5-d5ed-401e-b83d-d6e85d7d887c"
                description: "This function consists in decoding the video packets typically compressed using a video codec such as H264."
                parent: "880b2d92-4a5f-4b0c-9c54-4faa45efd4aa"
                port in "FIP 1" { id: "6012d367-0ebf-4a90-9fe0-bf5256d6e349" }
                port in "FIP 3" { id: "cfb3c498-9f99-4800-8a7b-078280ab479c" }
                port out "FOP 1" { id: "d4a397ea-4348-44fc-a60e-997d7f7d1961" }
            }
            function "Receive and Bufferize Video Packet" {
                id: "28b23885-f94c-44b9-8c2f-a2f52b379842"
                description: "As the video data packets arrive, they are buffered for a few seconds and then decoded for the playback to begin. Bufferizing the video packets reduces the risk of playback interruption due to network congestion."
                parent: "880b2d92-4a5f-4b0c-9c54-4faa45efd4aa"
                port in "FIP 1" { id: "86ca74ab-c628-43de-9e7a-4afa683b212e" }
                port out "FOP 1" { id: "e4bdc2e5-6e4c-452c-ae2a-0facd2b131ac" }
            }
        }
    }
    node "Cabin Screen Video Splitter" {
        id: "404eeafa-1b20-4162-893c-f0cfc1e836d6"
        port "PP 1" { id: "062e858e-6389-44c8-b8b2-b0c277b53360" }
        port "PP 2" { id: "9f2fee40-2a92-4a25-8348-32bc1b15b774" }
    }
    node "Cabin Video Display Unit" {
        id: "dcb6cca2-ea15-46f9-824f-530d40b4d5f9"
        port "PP 1" { id: "9fabd1fe-cfb0-459b-8766-1c2d1808f51c" }
        behavior_component "Cabin Screen SW" {
            id: "0c7d06de-bc6a-44d2-8b01-459cbdbbf69a"
            port out "CP 1" { id: "0f3c342b-3b6e-47ec-a457-0b53364dd844" }
            function "Display Video on Cabin Screen" {
                id: "23b2867d-b9cd-4b1d-8d40-7de838fc1313"
                description: "This function is in charge of displaying video frames on the cabin screens."
                parent: "3d159790-b7eb-46d7-90ac-d64017225cf0"
                port in "FIP 1" { id: "93584b75-90af-4ae3-8cf3-9d114e29bc51" }
                port out "FOP 1" { id: "60be30f4-42fb-49db-a655-03b4a263c95c" }
            }
        }
    }
    node "Passenger" {
        id: "ca1519c5-e748-43ee-a81b-474a31b7b5ab"
        actor: true
        port out "CP 1" { id: "618fdfc5-a78c-4b4f-8f30-329a10a43d89" }
        port in "CP 2" { id: "05ebe884-ca6f-4774-a9cf-2325c66890f2" }
        port in "CP 3" { id: "b837cd91-9cb0-44b7-96eb-8144d0dd5e82" }
        function "Command VOD Session" {
            id: "643033c9-5624-45f8-a1ce-b9bc4ddc2553"
            description: "This function is dedicated to the interactions in the context of the VOD service: Selection of movies (and languages) and control on these movies (play, pause, stop, etc.)."
            realizes: "bcb08500-a828-43b0-ac8b-ff4c7cb3b6b4"
            parent: "07256f31-32c7-42f5-b36b-bebd43492921"
            port in "FIP 1" { id: "0481c847-ee51-4f58-978d-6df70511d9a6" }
            port in "FIP 2" { id: "cfedb1a1-5b1f-4d52-981c-2ac113dda60a" }
            port in "FIP 3" { id: "f64e00fa-91a9-4476-ac74-7b10561f24db" }
            port out "FOP 1" { id: "575a3d6d-62ec-4ebd-b4e3-5a532ef4c654" }
            port out "FOP 2" { id: "8c49d811-8fa4-4bc2-9fee-67f90a72556e" }
            port out "FOP 3" { id: "fb75f445-21c6-438f-80af-c971c6a6b1fe" }
            port out "FOP 4" { id: "77c994fe-814c-4f97-a140-c1c53aa9c60a" }
            port out "FOP 5" { id: "32409033-8e8d-4dca-a9ad-5e25e99bd106" }
        }
        function "Listen to Audio Announcement" {
            id: "3159e079-92f0-4e22-8cef-53e8f447cd5f"
            description: "Audio announcements are information about the flight provided by the cabin crew or the captain (boarding, pre-flight announcements, captain's announcements, safety briefing, turbulence, take-off/ascent, descent/final, landing, etc.). The passenger can listen to these annoucements aither through the aircraft speakers either through the individual Seat TVs."
            realizes: "18da001e-dd8b-4041-ba4c-eb2211c624c4"
            parent: "07256f31-32c7-42f5-b36b-bebd43492921"
            port in "FIP 1" { id: "69dfc6ec-64ae-4d2e-8589-ef5ea1b429fb" }
            port in "FIP 2" { id: "c3ce3646-080b-4f30-8b61-748dda47c389" }
            port in "FIP 3" { id: "e6b181a4-f0b5-4228-9723-b3b07723db81" }
        }
        function "Select Passenger Service" {
            id: "a3605148-c80e-4635-9790-f068451a1b4a"
            description: "This function covers the selection by the passenger of a given IFE service (gaming, moving-map, VOD, etc.)."
            realizes: "63136eb0-622b-43cc-b4da-55b86ae281c1"
            parent: "07256f31-32c7-42f5-b36b-bebd43492921"
            port in "FIP1" { id: "7d60813d-f888-4f38-80d3-86f4a13a97db" }
            port out "FOP1" { id: "2b5f76fc-8e20-486e-a9ee-5ed5679d7848" }
            port out "FOP 2" { id: "6a30f4e8-1977-4e8c-84e4-8406ad336c78" }
        }
        function "Watch Imposed Movie on Cabin Screen" {
            id: "649d218e-6119-46b3-8c9e-377cf40e086d"
            description: "This function represents the fact that the passenger can watch a video on a cabin screen (typically located in the ceiling)."
            realizes: "295db719-853a-4cb6-9bbe-2e66b5ae9ace"
            parent: "07256f31-32c7-42f5-b36b-bebd43492921"
            port in "FIP3" { id: "767ebb84-54ce-49ad-8ff6-b0fed5dd10f3" }
            port in "FIP 2" { id: "37292878-12a2-416a-9965-295ac91fe143" }
            port in "FIP2" { id: "ca059337-a84b-465c-a85d-2c12ca2fd676" }
        }
        function "Watch Movie on Private Screen" {
            id: "be579285-807c-46ab-bcab-ae7efb19a8d6"
            description: "This function represents the fact that the passenger can watch a movie/video on a private screen (i.e. the in-seat TV)."
            realizes: "aeafffae-c0e8-45a3-9ad1-105825574ced"
            parent: "07256f31-32c7-42f5-b36b-bebd43492921"
            port in "FIP1" { id: "a51712b9-d5d1-47fe-a3bb-7e9c14a650f6" }
        }
    }
    node "Crew" {
        id: "a5782ec2-f546-49e5-98aa-88146d997d1d"
        actor: true
        port in "CP 1" { id: "19ba4056-ca11-4270-9182-3010597caf7f" }
        port out "CP 2" { id: "4a9a7aac-dd0d-41a5-92ac-ac11d5e9f39e" }
        function "Command Airline-Imposed Video Broadcast" {
            id: "292159f9-5917-4c0c-9ea3-2800431b3c48"
            description: "This function consists in triggering the play of pre-recorded airline videos. In particular, these imposed videos inclure pre-flight safety instructions (detailed explanation given before takeoff to airline passengers about the safety features of a commercial aircraft). Such imposed videos can also be commercial ads."
            realizes: "37c0f280-8012-4736-82df-4e9114fc61d4"
            parent: "e22a5c1a-1ee1-4bc4-8b34-b843f4bcf116"
            port in "FIP 1" { id: "a2e15cb9-a107-4f98-93b5-83c43c69f249" }
            port in "FIP 2" { id: "3d12db45-4b62-4183-8d19-50255e1e1936" }
            port in "FIP 3" { id: "e9961213-6c11-4ab7-a158-976d62a98196" }
            port out "FOP 1" { id: "3dc6a4f6-af6f-44e9-a2aa-61182b131341" }
            port out "FOP 2" { id: "3cb9c8c2-2fd3-40e8-ad4c-b7f584170310" }
            port out "FOP 3" { id: "8a52f7f8-8952-4dc6-9076-7e5138ced5f4" }
            port out "FOP 4" { id: "ec6e9133-bad0-48dd-899a-15d9cca59f00" }
        }
        function "Launch Pre-Flight Tests and Analyze Results" {
            id: "9b00bf4f-2f71-4f7a-9c51-4a84b84e18de"
            description: "This function consists in triggering a series of system auto-tests before each flight."
            realizes: "82445ac7-1b27-4c0c-bdff-7fda4e1e7496"
            parent: "e22a5c1a-1ee1-4bc4-8b34-b843f4bcf116"
        }
    }
    node "Aircraft" {
        id: "8aba6e60-54e0-401a-bc8b-c86090e030d0"
        actor: true
        function "Play Audio on Cabin Aircraft Speakers" {
            id: "9c247f2d-ed9b-43e8-921a-d1fdbb331ab1"
            description: "This function receives digital audio streams and play them on the aricraft speakers."
            realizes: "c5e641ab-a062-4856-87ae-e675061c3eb5"
            parent: "75c66998-1426-4629-94e6-b01fafd37579"
            port in "FIP 1" { id: "e19a8032-588d-4e53-86ad-fce3c3e9dc3d" }
            port out "FOP1" { id: "e18a66b6-dd5c-4b04-a047-3b0f9beeeb1e" }
        }
        function "Provide Electrical Power" {
            id: "4c237128-0cf3-4074-8af4-5e63e4994e02"
            description: "This function is responsible for providing power consumption profiles. According to different flying conditions and phases, the power made available to the IFE system may differ."
            realizes: "4d1c1e5c-4c96-41fc-a4a5-4bd74085f320"
            parent: "75c66998-1426-4629-94e6-b01fafd37579"
            port out "FOP 1" { id: "68d57242-548d-41c4-879d-3a68de004e23" }
        }
        function "Provide Exterior-View Videos" {
            id: "6a02a1d5-5b6b-4ea7-8494-4afca0a68e4e"
            description: "This functions relays the videos captured external cameras, typically located on the tail or the nose of the plane."
            realizes: "9ae12cc6-8bb4-4f99-ad87-8a30a22b919b"
            parent: "75c66998-1426-4629-94e6-b01fafd37579"
        }
        function "Provide Navigation Data" {
            id: "39b3d893-7961-4f0e-9b35-6090100f2c37"
            description: "Navigation is the determination of position and direction on or above the surface of the Earth. Avionics can use satellite-based systems (such as GPS and WAAS), ground-based systems (such as VOR or LORAN), or any combination thereof. The IFE system requires this data to run the moving-map service."
            realizes: "ebce47df-f06c-4eea-b367-b828df4b44fa"
            parent: "75c66998-1426-4629-94e6-b01fafd37579"
        }
        function "Provide Satellite Communication Means" {
            id: "921af19b-4f01-46af-b01e-f6cd6bbfe3c1"
            description: "This function consists in providing satellite network, typically for telephony or in-cabin wifi systems."
            realizes: "55e3b1b8-dea2-4fbf-b78f-0f270009b56a"
            parent: "75c66998-1426-4629-94e6-b01fafd37579"
        }
        function "Send Audio Announcements" {
            id: "820d4407-7206-4075-91fc-d76ce083056e"
            description: "Audio announcements include: boarding, pre-flight announcements, captain's announcements, safety briefing, turbulence, take-off/ascent, descent/final, landing, etc."
            realizes: "c6ed6258-be4c-4784-a6e4-3e95f9f22dc7"
            parent: "75c66998-1426-4629-94e6-b01fafd37579"
            port out "FOP 5" { id: "95ffe104-c558-40a0-8bcf-ae7cb7c6f3d4" }
        }
        function "Send Decompression Notification" {
            id: "75e214e4-74f8-4c44-97f9-f328b8ac3f97"
            description: "If ever a decompression occurs, a notification is sent to the IFE system (which will automatically swtich to \"Halted\" mode."
            realizes: "e01b39c5-f018-4d9c-af36-c83f1b9a8a81"
            parent: "75c66998-1426-4629-94e6-b01fafd37579"
            port out "FOP 1" { id: "a1277e72-90ac-45ef-82c6-84dd5009b55c" }
        }
    }
    node "Maintenance operator" {
        id: "f459f10a-d8ae-4938-b3e1-4eb294ec1c32"
        actor: true
        function "Configure the System" {
            id: "03df466f-aebe-4fec-8617-e82e6bf9ed5d"
            description: "[Not covered in this version of the model]"
            realizes: "9307409a-5b76-4a96-876c-85468c60e376"
            parent: "6093275a-6ff1-49a6-9384-dea2bab8510d"
        }
        function "Launch Tests and Analyze Results" {
            id: "d8b3f902-a4ed-4c9e-a650-e5c159c8d74f"
            description: "[Not covered in the version of the model] This function consists in triggering system tests, typically during maintenance phases."
            realizes: "05aefe2e-d873-4fb0-8ad7-4dafc46c764e"
            parent: "6093275a-6ff1-49a6-9384-dea2bab8510d"
        }
        function "Set Passenger Service Authorization" {
            id: "a8b9e953-b751-4991-b90e-e3ec251f65f0"
            description: "The airline company may have specific business strategies: According to the class they are travelling in, passengers may benefit from different IFE services. This function consists in uploading a profile that describes the authorization schemes."
            realizes: "9d3d6fe7-16df-4382-9d49-492e4cd2b73f"
            parent: "6093275a-6ff1-49a6-9384-dea2bab8510d"
            port out "FOP 1" { id: "00275812-33af-4bf9-a241-0fc43e5af084" }
        }
        function "Update Media Content" {
            id: "eab9aed9-2576-4d7b-ad86-021e79aac4e1"
            description: "This function consists in updating the IFE media server with new content. Using a specific device, the ground operator can for example updload new movies/music and remove others."
            realizes: "d5254d45-3946-431f-8162-a28a3f2263a7"
            parent: "6093275a-6ff1-49a6-9384-dea2bab8510d"
            port out "FOP 1" { id: "016e3f37-0fa5-4eb5-9dde-1ad8e0add758" }
        }
    }
    function "Entertain with IFE System" {
        id: "07256f31-32c7-42f5-b36b-bebd43492921"
        description: "This function is a high-level function gathering all the what is performed by the passenger actor."
        realizes: "cf623512-1614-4548-9b4c-4c0e614c4516"
    }
    function "IFE System (2)" {
        id: "c75359fb-2271-4499-8199-c3e1565697a7"
        description: "This function is a high-level function gathering all the what is performed by the IFE system."
        realizes: "145e28dc-9fbe-4f01-a378-5af8cbe8e2d7"
        function "Distribute data" {
            id: "03b0cf66-8fd1-4867-bec3-bc49814211a9"
            description: "This function gathers network management functions."
            function "Setup Network" {
                id: "7f4ecc96-3b56-4470-a35b-ce764d0b4e1f"
                description: "This function groups a set of functions in charge of preparting and monitoring the IFE network."
            }
        }
        function "Manage Aircraft Interface" {
            id: "1da11685-85e8-4ea3-a6e8-e7cfe33111d4"
            description: "This functions regroups all the functions that interact directly with the aircraft."
            realizes: "c268e3f1-3cbc-4e62-8ae2-75975bc4481f"
        }
        function "Manage Audio Video Diffusion" {
            id: "cb0426b2-9cab-4a4d-9e7b-89a13a57be2e"
            description: "This function is responsible for broadcasting audio and video streams, should they be VOD movies, imposed videos, on-demand or imposed music. This function also covers the transmission of audio announcements (coming from the aircraft) and the IFE passenger service interruptions triggered by these announcements."
            realizes: "95495bbc-987c-4b17-95d0-9aa5a87f0344"
            function "Broadcast Stored Audio and Video Streams" {
                id: "cf0384ae-550f-40a4-a860-7d081b24a01e"
                description: "This function is responsible for broadcasting existing audio and video media. Transmitting audio and video streams means: - Sending them to each passenger (on Seat TVs) - Sending them to the cabin screens"
                realizes: "d4ee89c8-dcf8-4fa5-b276-0afa03d409c9"
            }
            function "Display Video and Play Audio" {
                id: "3d159790-b7eb-46d7-90ac-d64017225cf0"
                description: "This function aggregates different display and media playing functions, which are typically allocated to different equiments: Seat TVs, Cabin Screens, etc."
                realizes: "cbe07842-a67d-488a-bf29-5b46d3369c15"
                function "Play Audio on Seat TV" {
                    id: "6f47362b-60ef-451f-8a00-005ebd4fd010"
                    description: "This function receives audio packets and restitutes the corresponding audio stream through the Seat TV."
                }
                function "Play Video Stream on Cabin Screen" {
                    id: "880b2d92-4a5f-4b0c-9c54-4faa45efd4aa"
                    description: "This function receives video packets and restitutes the corresponding video through the cabin screens. The audio track associated to the video is forwarded to the aircraft through another function."
                    realizes: "2632b523-b577-48c5-afba-3eb4ccde3254"
                }
                function "Play Video Stream on Seat TV" {
                    id: "d8e4de58-aebd-43de-85d6-b7e0011cf619"
                    description: "This function receives video packets and restitutes the corresponding video through the Seat TV."
                    realizes: "7b604e03-85be-4d93-8035-8a84ef8086db"
                }
            }
            function "Process Audio Video Requests" {
                id: "8e79917f-4323-4040-b8c5-64c5e42c2a3b"
                description: "This function groups sub functions that are responsible for tranforming the end-user (passenger, cabin crew) selections in applicative commands."
                realizes: "a8452b58-3135-457b-8d85-59bd23240297"
            }
        }
        function "Manage Passenger Services Lifecycle" {
            id: "cf7dca25-2250-4339-9501-eaf3557a644c"
            description: "This function is responsible for controling the launch of services such as VOD, gaming, moving-map, etc. It determines the availability of the services given the airline company authorization scheme and the constraints received from the aircraft. This function provides means to select among a predefined list of services."
            realizes: "11f3e0e8-a78d-4962-bed8-9672b1167331"
            function "Launch Service" {
                id: "2e06d355-e036-40f8-9f84-074d413e7b07"
                description: "This function gathers the two functions in charge of managing the activation and de-asctivation of passenger services."
            }
        }
        function "Provide Access to Digital Media" {
            id: "54b31979-1ec1-49c5-81e0-512fd1aec704"
            description: "This functions gathers the functions related to the management of the digital media content of the IFE: load and storage."
            realizes: "2a6d57b4-97f3-4344-b645-cefcc9462708"
        }
        function "Run Services" {
            id: "196af4c5-87f3-44b7-8cc6-74d91d4d1c9a"
            description: "This function regroups all IFE services running functions."
            realizes: "355a6d2b-8c53-46f8-8c0b-38356b4d4271"
            function "Run Cabin Intercommunication Service" {
                id: "46a33304-1338-4fc0-a9cb-dc3fe19e5ed4"
                description: "Cabin intercommunication is a cabin crew service. This function gathers all functions in charge of managing the interactions of the cabin crew related to imposed videos (selection of videos, play/pause/stop controls, display of video data, etc.)."
                realizes: "81d017bc-85ca-4bcd-acc9-51423542c61e"
            }
            function "Run Video-On-Demand Service" {
                id: "f9353538-3fcb-4697-8591-daf861b7a164"
                description: "VOD Service is a passenger service. This function gathers all functions in charge of managing the VOD-related interactions of the passenger (selection of movies, play/pause/stop controls, display of movie data, etc.)."
                realizes: "7a8c51b2-c42f-49cd-a003-752d0de0b4da"
            }
        }
    }
    function "Perform Cabin Management Activities" {
        id: "e22a5c1a-1ee1-4bc4-8b34-b843f4bcf116"
        description: "This function is a high-level function gathering all the what is performed by the ground operator actor."
        realizes: "bca11666-f58a-41e4-9a16-5964771b55b0"
    }
    function "Perform Maintenance, Configuration and Test Operations" {
        id: "6093275a-6ff1-49a6-9384-dea2bab8510d"
        description: "This function is a high-level function gathering all the what is performed by the aircraft actor."
        realizes: "bf6388c4-3549-46a6-af08-48412581967e"
    }
    function "Perform remote diagnostic" {
        id: "f46f4ddc-51cc-4b2c-ba6c-d9e8c0bae8ea"
        function "PF8" {
            id: "29c4014e-7db7-4660-9f3e-e0a8a37e0090"
            port in "FIP 1" { id: "83043aff-5c50-4a41-9312-9949d5e53cac" }
            port out "FOP 1" { id: "f412d613-7253-4ad7-aa4c-d0efbf09a74c" }
        }
        function "PF9" {
            id: "52d67cae-5ad2-4c5d-b4bf-e68d308c73b3"
            port in "FIP 1" { id: "60ff7e8c-32bb-493c-90c1-3fe7a6ab05f3" }
        }
    }
    function "Provide Aircraft Information, Commands and Means" {
        id: "75c66998-1426-4629-94e6-b01fafd37579"
        description: "This function is a high-level function gathering all the what is performed by the aircraft actor."
        realizes: "cd4b1bad-e12e-45eb-96fd-dfec5b483457"
    }
    function "REC Unit Network Adapter" {
        id: "71f18377-d79b-4619-a537-cfc07ecb8931"
        description: "This function is a pure technical container for the functions belonging to the REC \"Unit Netwrok Adatper\""
    }
    functional_exchange "Digital Media" { id: "21b77a35-529e-4bce-98eb-8c6e8e2fc01a" from: "16904687-4018-46cc-a0ec-c415f6d2bc1d.FOP 1" to: "0378eba1-fb7b-4b4d-b39d-7a14ce4e5f4e.FIP 4" }
    functional_exchange "Network Configuration" { id: "0e2392a9-e101-41a6-8263-d41ea21acb48" from: "3fcaa02a-0321-4be3-ab79-b2547d47749b.FOP 1" to: "e24662c4-47b2-445f-9d18-7214fca7013e.FIP 1" }
    functional_exchange "Network Configuration (2)" { id: "c091c3c7-137a-4601-93d0-4e8d015c63d5" from: "6aa6df97-e744-4b1b-962f-66874bfabbaa.FOP 1" to: "e24662c4-47b2-445f-9d18-7214fca7013e.FIP 1" }
    functional_exchange "Network Test Sequences" { id: "6a95a2a2-2cab-45f3-8ba2-877ab15a158e" from: "e323d7b2-2bb2-4ffe-a1b5-265c98befe28.FOP 1" to: "ccfb7dc0-c861-44db-95ea-0a216b9ec643.FIP 1" }
    functional_exchange "Network Configuration (3)" { id: "4a0118f2-17ee-48ca-a073-67fa91727d09" from: "6aa6df97-e744-4b1b-962f-66874bfabbaa.FOP 1" to: "84c8b355-ee9b-48dd-aca8-17b0c4e477c8.FIP 1" }
    functional_exchange "Network Configuration (4)" { id: "61395f64-96f1-48a4-9044-7d77bc58e354" from: "6aa6df97-e744-4b1b-962f-66874bfabbaa.FOP 1" to: "c5503a00-fae8-439d-90a7-6b8644d2f609.FIP 1" }
    functional_exchange "Network Configuration (5)" { id: "78a8db68-010f-4dcf-830b-e44ba8bf55e9" from: "3fcaa02a-0321-4be3-ab79-b2547d47749b.FOP 1" to: "c5503a00-fae8-439d-90a7-6b8644d2f609.FIP 1" }
    functional_exchange "Network Configuration (6)" { id: "ae3713fd-e78c-4a90-b323-2438d7abfcd0" from: "3fcaa02a-0321-4be3-ab79-b2547d47749b.FOP 1" to: "84c8b355-ee9b-48dd-aca8-17b0c4e477c8.FIP 1" }
    functional_exchange "Network Test Sequences (2)" { id: "f3664761-31ae-42ce-9eaa-49e1080b3003" from: "e323d7b2-2bb2-4ffe-a1b5-265c98befe28.FOP 1" to: "238797f1-c012-446a-9b79-e94cf88bd180.FIP 1" }
    functional_exchange "Network Test Sequences (3)" { id: "3fe443c0-69f2-4c9c-9e7c-c228b6a12077" from: "e323d7b2-2bb2-4ffe-a1b5-265c98befe28.FOP 1" to: "b2fe1c12-86f0-4c40-9986-d349c2a09496.FIP 1" }
    functional_exchange "VOD Movie Control Command" { id: "f0fb96ca-05e7-4119-99ff-b08e8184206e" from: "ac5be58c-aa92-4247-8c8f-9867ddbd6289.FOP 2" to: "d53e6dd4-3926-4b25-80d1-14553a472f9e.FIP 2" exchange_items: ["d3b096f5-9eb4-4c4a-b0af-946a71992c54"] category: "VOD Movie Control Commands" }
    functional_exchange "Imposed Video Control Command" { id: "588d634d-b2c8-4632-8276-000f5b97dace" from: "737aebcf-1f00-40ac-8e03-df8bc6596159.FOP 3" to: "d53e6dd4-3926-4b25-80d1-14553a472f9e.FIP 3" }
    functional_exchange "Broadcast Data" { id: "92d7c48e-f677-4bab-9bdb-ee7bb1eb7a28" from: "d53e6dd4-3926-4b25-80d1-14553a472f9e.FOP 1" to: "98960be5-3e8f-4743-ab70-805645472512.FIP 2" exchange_items: ["03c86c27-a965-4b3a-b641-e1969d4c7261"] }
    functional_exchange "Audio Video Packets" { id: "cf3c82cc-09dd-4a23-b675-564bf63724a2" from: "98960be5-3e8f-4743-ab70-805645472512.FOP 1" to: "2dfb0a52-7568-4903-9c78-1e462848f868.FIP 1" exchange_items: ["b65b0dee-34a9-4397-9626-e8095e348482"] }
    functional_exchange "Video Packets" { id: "c2fe2129-eb3c-485f-b323-098e89147d3d" from: "ecc7d466-f9cb-4be1-a099-b8449146cf18.FOP 1" to: "9d3a7965-d967-42e2-8f25-c0113abaf9ff.FIP 1" exchange_items: ["911f07f3-4c41-4351-aff4-1f1a84d9417a"] }
    functional_exchange "Video Data Packet" { id: "8b0dd14d-3523-49dd-95ee-18973eb41f1a" from: "28b23885-f94c-44b9-8c2f-a2f52b379842.FOP 1" to: "c91204f5-d5ed-401e-b83d-d6e85d7d887c.FIP 1" }
    functional_exchange "Audio Packets" { id: "6527f025-6288-43d3-af86-c19c306f30dc" from: "9155e7ae-b4c3-4f06-b8b2-0ec90045ac8e.FOP 1" to: "07823fa4-38c9-43fe-8b4b-8aa5588b7c46.FIP 1" exchange_items: ["cb931585-e5a5-48ba-8ac8-cf3b13fffe33"] }
    functional_exchange "Imposed Video Status" { id: "05c4227e-c46c-4a4b-b9e9-9b822a5ec038" from: "e8af73d3-bacb-4222-b387-f6b0bd4d13a1.FOP 1" to: "72f0a4da-866e-4136-8ca7-bfc85db05005.FIP 1" }
    functional_exchange "Played Video" { id: "09e20303-3d21-4966-b666-0b64a67e5089" from: "c91204f5-d5ed-401e-b83d-d6e85d7d887c.FOP 1" to: "23b2867d-b9cd-4b1d-8d40-7de838fc1313.FIP 1" }
    functional_exchange "Packetized Video Packets" { id: "1ff105b1-55b0-4eb8-a72d-e9f12d08fd9e" from: "4c361e00-d6d0-4acf-a523-1de11acf7797.FOP 1" to: "ecc7d466-f9cb-4be1-a099-b8449146cf18.FIP 1" exchange_items: ["911f07f3-4c41-4351-aff4-1f1a84d9417a"] }
    functional_exchange "Packetized Audio Stream" { id: "4f7e66d9-60ac-41d2-a4c3-2f1abe5e821c" from: "4c361e00-d6d0-4acf-a523-1de11acf7797.FOP 2" to: "9155e7ae-b4c3-4f06-b8b2-0ec90045ac8e.FIP 2" exchange_items: ["cb931585-e5a5-48ba-8ac8-cf3b13fffe33"] }
    functional_exchange "Decoded Audio Stream" { id: "3d9cf819-f43e-4e83-bec0-b37ac96faa40" from: "07823fa4-38c9-43fe-8b4b-8aa5588b7c46.FOP 1" to: "a84dff3b-2158-41ed-b96c-0b8ace0bea3b.FIP 2" exchange_items: ["cb931585-e5a5-48ba-8ac8-cf3b13fffe33"] }
    functional_exchange "Video Frames" { id: "e10b56a6-a7f3-43af-8352-98fdcbfabb4e" from: "9d3a7965-d967-42e2-8f25-c0113abaf9ff.FOP 1" to: "bd383ff2-258a-41dd-973f-07f7aa30da7a.FIP 1" exchange_items: ["a4ecc90d-72fb-486d-8de2-90d5473df459"] }
    functional_exchange "Sync Metadata" { id: "de4f60ad-5acc-42bd-ad6b-2cacaf830bfb" from: "1289a361-a3e8-416b-a9e0-e4fcd528d9e3.FOP 1" to: "07823fa4-38c9-43fe-8b4b-8aa5588b7c46.FIP 2" }
    functional_exchange "Sync Metadata (2)" { id: "355c5b13-8a3e-4c5f-b186-ebe6ea774140" from: "1289a361-a3e8-416b-a9e0-e4fcd528d9e3.FOP 2" to: "9d3a7965-d967-42e2-8f25-c0113abaf9ff.FIP 2" }
    functional_exchange "Packetized Video Stream" { id: "bc4ac210-f2e5-4332-b548-69f52ec5ff64" from: "2dfb0a52-7568-4903-9c78-1e462848f868.FOP 2" to: "28b23885-f94c-44b9-8c2f-a2f52b379842.FIP 1" }
    functional_exchange "Pause-Stop Notification" { id: "7f7b6497-509f-416f-a714-a2e3a6065e76" from: "d53e6dd4-3926-4b25-80d1-14553a472f9e.FOP 2" to: "ecc7d466-f9cb-4be1-a099-b8449146cf18.FIP 2" }
    functional_exchange "Pause-Stop Notification (2)" { id: "4dc4747f-d2a3-4f50-b290-790bf206ea96" from: "d53e6dd4-3926-4b25-80d1-14553a472f9e.FOP 2" to: "c91204f5-d5ed-401e-b83d-d6e85d7d887c.FIP 3" }
    functional_exchange "Pause-Stop Notification (3)" { id: "f9231b85-ee09-49bc-b407-dd66190a7b82" from: "d53e6dd4-3926-4b25-80d1-14553a472f9e.FOP 2" to: "e8af73d3-bacb-4222-b387-f6b0bd4d13a1.FIP 4" }
    functional_exchange "Imposed Video Playing Status" { id: "303c4e84-7d9e-4c13-96d3-c898826da1b9" from: "d53e6dd4-3926-4b25-80d1-14553a472f9e.FOP 4" to: "e8af73d3-bacb-4222-b387-f6b0bd4d13a1.FIP 1" }
    functional_exchange "Packetized Audio Stream (2)" { id: "664c04f7-85af-4aef-a7a7-cb4f31832495" from: "80a8d0f6-ca9e-486f-9ac3-0c524de61281.FOP 3" to: "9155e7ae-b4c3-4f06-b8b2-0ec90045ac8e.FIP 1" }
    functional_exchange "Packetized Audio Video Stream" { id: "5749f82b-2323-4e33-9fa7-3253d6fdc1ee" from: "2dfb0a52-7568-4903-9c78-1e462848f868.FOP 1" to: "4c361e00-d6d0-4acf-a523-1de11acf7797.FIP 1" exchange_items: ["b65b0dee-34a9-4397-9626-e8095e348482"] }
    functional_exchange "Packetized Audio Stream (3)" { id: "86900f67-a76d-4996-91ce-e94dc33333be" from: "2dfb0a52-7568-4903-9c78-1e462848f868.FOP 3" to: "ccf41abe-2a9d-4f50-bc3d-643397b7a338.FIP 2" }
    functional_exchange "Stream Header" { id: "d66928f0-fe85-4e75-ab12-a750b1c832cb" from: "98960be5-3e8f-4743-ab70-805645472512.FOP 2" to: "1289a361-a3e8-416b-a9e0-e4fcd528d9e3.FIP 2" exchange_items: ["b1ca4bf5-1b54-4f5c-a385-9514227695d3"] }
    functional_exchange "VOD Movie Data Request" { id: "67a82bc3-0c6a-457b-8558-333cbb724512" from: "0c8009a9-0524-4445-a9c8-f0295467f62a.FOP 3" to: "dcbf11ee-150a-4afa-8e06-3f0ab9aee27a.FIP 1" category: "Media Requests" }
    functional_exchange "VOD Movie Description Request" { id: "70ec2604-a2d3-48c3-9594-a162d2ee8182" from: "0c8009a9-0524-4445-a9c8-f0295467f62a.FOP 4" to: "dcbf11ee-150a-4afa-8e06-3f0ab9aee27a.FIP 2" category: "Media Requests" }
    functional_exchange "VOD Movie List Request" { id: "b4e7ccf9-a718-439f-9ac3-56494cd432d2" from: "0c8009a9-0524-4445-a9c8-f0295467f62a.FOP 5" to: "dcbf11ee-150a-4afa-8e06-3f0ab9aee27a.FIP 3" category: "Media Requests" }
    functional_exchange "Available VOD Movies List" { id: "7739f8e7-2b55-4805-847e-ecdba2a46b46" from: "dcbf11ee-150a-4afa-8e06-3f0ab9aee27a.FOP 1" to: "c7edaab3-11bc-4409-b300-13c4b4192742.FIP 4" category: "Media Content" }
    functional_exchange "Movie Description" { id: "7611e13a-7585-4400-9a0f-58adbe6c9fed" from: "dcbf11ee-150a-4afa-8e06-3f0ab9aee27a.FOP 2" to: "c7edaab3-11bc-4409-b300-13c4b4192742.FIP 1" category: "Media Content" }
    functional_exchange "Movie Picture" { id: "d1520556-1979-4d95-843d-493272c4a36c" from: "dcbf11ee-150a-4afa-8e06-3f0ab9aee27a.FOP 3" to: "c7edaab3-11bc-4409-b300-13c4b4192742.FIP 3" category: "Media Content" }
    functional_exchange "Available Languages List" { id: "4ab2fec4-58d0-4f7b-a219-0f8ff8d5ee7a" from: "dcbf11ee-150a-4afa-8e06-3f0ab9aee27a.FOP 4" to: "c7edaab3-11bc-4409-b300-13c4b4192742.FIP 2" category: "Media Content" }
    functional_exchange "VOD Movie Interruption Command" { id: "21b38194-329a-4371-82cd-5d6e8085d550" from: "241dcfae-1880-45f9-bf2c-32b0cbe736b7.FOP 2" to: "8daae3ab-944b-4ade-9896-0d51c62ef3e8.FIP 1" }
    functional_exchange "VOD Movie List Request (2)" { id: "243d3251-c6e2-4973-9810-3dd8c0c39aae" from: "241dcfae-1880-45f9-bf2c-32b0cbe736b7.FOP 3" to: "dcbf11ee-150a-4afa-8e06-3f0ab9aee27a.FIP 8" }
    functional_exchange "Imposed Video Selection Command" { id: "ee4691b3-538b-4812-9982-9c068f93a1ae" from: "9b798c7d-2d5a-43f9-867d-b26c3f4c1337.FOP 2" to: "1cda3b05-ee17-4912-b875-40bc4ddca86b.FIP 1" }
    functional_exchange "Available Imposed Video List" { id: "8e0d22ac-7506-4a2e-8bc2-7a97f94fa52b" from: "1cda3b05-ee17-4912-b875-40bc4ddca86b.FOP2" to: "cebc5976-81d6-4718-b1f0-9f8b7687ceeb.FIP1" category: "Media Content" }
    functional_exchange "Service Stored State" { id: "0b3cab56-9c1e-418b-a5ed-049179978bb9" from: "8835c15a-0eb0-4d20-83d2-a15f8c0bd005.FOP 3" to: "8cc1e6f5-dbe2-4629-8a43-b60eef771a8f.FIP 2" }
    functional_exchange "Service Status Request" { id: "010959eb-bbfe-40fa-a2fa-fa2683769b48" from: "8cc1e6f5-dbe2-4629-8a43-b60eef771a8f.FOP 1" to: "8835c15a-0eb0-4d20-83d2-a15f8c0bd005.FIP 1" }
    functional_exchange "Passenger Homepage Display Command" { id: "06c03c7d-5648-44b2-a6b9-336377d27d8d" from: "9d76fefa-2f89-4272-8b1d-785d05601ce6.FOP 1" to: "83b4863a-7795-4911-aeef-2938443cce3d.FIP 1" }
    functional_exchange "VOD Service Activation Command" { id: "46ffbaa1-f974-4536-bae1-3d0de536ec18" from: "9d76fefa-2f89-4272-8b1d-785d05601ce6.FOP1" to: "8cc1e6f5-dbe2-4629-8a43-b60eef771a8f.FIP1" }
    functional_exchange "Available Passenger Service Request" { id: "dbe313f4-8d74-44d6-8384-bf3d1d08f0c7" from: "83b4863a-7795-4911-aeef-2938443cce3d.FOP 2" to: "88a4d6d0-fcf2-4c03-be5e-e6a23c727bf4.FIP 3" }
    functional_exchange "Passenger Available Services List" { id: "3b738b12-a93b-4d8c-b015-9a8da58a8494" from: "88a4d6d0-fcf2-4c03-be5e-e6a23c727bf4.FOP 1" to: "83b4863a-7795-4911-aeef-2938443cce3d.FIP 2" }
    functional_exchange "Passenger Available Services List (2)" { id: "4d55bc07-f92d-4fca-9c67-faaa929b45d2" from: "88a4d6d0-fcf2-4c03-be5e-e6a23c727bf4.FOP 1" to: "8cc1e6f5-dbe2-4629-8a43-b60eef771a8f.FIP 4" }
    functional_exchange "Service Deasctivation Command" { id: "9c35a329-9aae-47e9-8536-b4edbac5d3e5" from: "9d76fefa-2f89-4272-8b1d-785d05601ce6.FOP 2" to: "8835c15a-0eb0-4d20-83d2-a15f8c0bd005.FIP 2" }
    functional_exchange "Service Interruption Command" { id: "ac16ed40-d6fd-4df7-84f9-202ec5c8af13" from: "3f69f8bd-704d-438b-8f05-fc7d56363811.FOP1" to: "241dcfae-1880-45f9-bf2c-32b0cbe736b7.FIP 3" category: "Service Activation / Interruption" }
    functional_exchange "Service Interruption Command (2)" { id: "2903db78-bb66-4227-b884-69222e103795" from: "737aebcf-1f00-40ac-8e03-df8bc6596159.FOP 4" to: "3f69f8bd-704d-438b-8f05-fc7d56363811.FIP 3" category: "Service Activation / Interruption" }
    functional_exchange "VOD Movie Status" { id: "42b917a3-2f0a-46a7-acbe-d7a1c4c4354d" from: "8daae3ab-944b-4ade-9896-0d51c62ef3e8.FOP 3" to: "241dcfae-1880-45f9-bf2c-32b0cbe736b7.FIP 1" }
    functional_exchange "Service Interruption Command (3)" { id: "f8152993-9e77-4021-a05f-71b4c6a5c843" from: "80a8d0f6-ca9e-486f-9ac3-0c524de61281.FOP 2" to: "3f69f8bd-704d-438b-8f05-fc7d56363811.FIP 2" category: "Service Activation / Interruption" }
    functional_exchange "Audio Stream" { id: "19ae7d7c-e19b-402e-a346-c4fec926cc2e" from: "2788898f-6daf-43c1-bca8-6ab484ac24c6.FOP 1" to: "80a8d0f6-ca9e-486f-9ac3-0c524de61281.FIP 1" }
    functional_exchange "Audio Interruption Notification" { id: "18fcdf21-f3b3-45ac-b0cb-966394a90b7f" from: "3f69f8bd-704d-438b-8f05-fc7d56363811.FOP 2" to: "068dc2db-3e29-421d-84be-8444ce8acc9c.FIP 1" }
    functional_exchange "FE1" { id: "c97cc795-07dd-45b2-b9bc-b875bb006d01" from: "88688938-d911-4317-b36e-266cef11a7f3.FOP 1" to: "fd6de739-ae93-4e5f-acaf-4e8fb8f0b2e5.FIP 1" }
    functional_exchange "FE2.1" { id: "904bdd67-1ec4-4243-85bf-b1ebe1b8db6e" from: "fd6de739-ae93-4e5f-acaf-4e8fb8f0b2e5.FOP 1" to: "1c49d7b2-bb7a-4939-86c1-a73c0822fdf5.FIP 1" }
    functional_exchange "FE2.2" { id: "89ac0547-79d3-41d4-a655-ba8d9da2468a" from: "fd6de739-ae93-4e5f-acaf-4e8fb8f0b2e5.FOP 2" to: "79218945-69c6-4bc6-8aa6-9d67f537492d.FIP 1" }
    functional_exchange "FE3.1" { id: "8693abc6-1876-4fcb-870e-6f860cb09170" from: "1c49d7b2-bb7a-4939-86c1-a73c0822fdf5.FOP 1" to: "0012d480-7be3-452f-9872-3a0d8868e2f6.FIP 1" }
    functional_exchange "FE3.2" { id: "84ea99c3-18a2-45ca-8ccc-14d43d9b6198" from: "79218945-69c6-4bc6-8aa6-9d67f537492d.FOP 1" to: "0012d480-7be3-452f-9872-3a0d8868e2f6.FIP 2" }
    functional_exchange "FE4" { id: "c3fb0b2f-490d-478f-9ed9-f2a40f08da16" from: "0012d480-7be3-452f-9872-3a0d8868e2f6.FOP 1" to: "1291c3c4-170d-4fcc-9b59-5b0961cd4a2d.FIP 1" }
    functional_exchange "FE5" { id: "82ae3980-a9fc-4922-a2a8-09de7b1461e4" from: "1291c3c4-170d-4fcc-9b59-5b0961cd4a2d.FOP 1" to: "71147664-0fcd-401f-937a-bdc76216924d.FIP 1" }
    functional_exchange "FE6" { id: "f4101c8b-3a2f-4fb9-a4bf-9684035adba5" from: "71147664-0fcd-401f-937a-bdc76216924d.FOP 1" to: "62f152bd-6017-4828-9034-7af3c387f83a.FIP 1" }
    functional_exchange "FE7" { id: "b10feab0-81b4-4cdf-87a5-e06457ea6873" from: "62f152bd-6017-4828-9034-7af3c387f83a.FOP 1" to: "29c4014e-7db7-4660-9f3e-e0a8a37e0090.FIP 1" }
    functional_exchange "FE8" { id: "8fc4392b-c6b5-4f4d-a50d-ff2310de47ac" from: "29c4014e-7db7-4660-9f3e-e0a8a37e0090.FOP 1" to: "52d67cae-5ad2-4c5d-b4bf-e68d308c73b3.FIP 1" }
    functional_exchange "Displayed Imposed Video Description" { id: "d5ba5cfa-5020-4b2c-885d-e33eb3afebb4" from: "cebc5976-81d6-4718-b1f0-9f8b7687ceeb.FOP2" to: "292159f9-5917-4c0c-9ea3-2800431b3c48.FIP 1" category: "Displayed Imposed Video Data" }
    functional_exchange "Displayed Audio Video" { id: "bb5c3c57-edd1-43d8-86d0-9e46e72ce11d" from: "bd383ff2-258a-41dd-973f-07f7aa30da7a.FOP 1" to: "be579285-807c-46ab-bcab-ae7efb19a8d6.FIP1" category: "Played Media" }
    functional_exchange "Passenger Homepage Selection" { id: "e207d8b6-c76d-494d-9519-b7f0cac18956" from: "a3605148-c80e-4635-9790-f068451a1b4a.FOP 2" to: "9d76fefa-2f89-4272-8b1d-785d05601ce6.FIP 1" category: "Passenger Service Selection" }
    functional_exchange "VOD Service Selection" { id: "a91fa283-42a3-4415-9274-9d09244ab3e6" from: "a3605148-c80e-4635-9790-f068451a1b4a.FOP1" to: "9d76fefa-2f89-4272-8b1d-785d05601ce6.FIP1" category: "Passenger Service Selection" }
    functional_exchange "VOD Movie Play Command" { id: "37794c77-aa6a-46ee-acbb-f75b65f491aa" from: "0c8009a9-0524-4445-a9c8-f0295467f62a.FOP 6" to: "ac5be58c-aa92-4247-8c8f-9867ddbd6289.FIP 3" exchange_items: ["1cd707ba-fe65-4e3c-9fff-21966d4e5716"] category: "VOD Movie Control Commands" }
    functional_exchange "Media Files" { id: "e2e69198-7a61-4ad1-bf57-2d01856924c1" from: "0378eba1-fb7b-4b4d-b39d-7a14ce4e5f4e.FOP 1" to: "d53e6dd4-3926-4b25-80d1-14553a472f9e.FIP 1" exchange_items: ["a34141d6-27d6-4e31-9994-828ee1560ded"] category: "Media Content" }
    functional_exchange "Displayed Movies List" { id: "3cbdd9e7-076b-49e2-a439-ac48669d54fe" from: "c7edaab3-11bc-4409-b300-13c4b4192742.FOP 3" to: "643033c9-5624-45f8-a1ce-b9bc4ddc2553.FIP 2" category: "Displayed VOD Movie Data" }
    functional_exchange "Audio Signal" { id: "10adb5f8-049d-4ea2-b5fa-87a1e23a6b6b" from: "9c247f2d-ed9b-43e8-921a-d1fdbb331ab1.FOP1" to: "649d218e-6119-46b3-8c9e-377cf40e086d.FIP3" }
    functional_exchange "Passenger Service Authorization Profile" { id: "ca58ed40-1dcf-4875-84e0-d4548351eea1" from: "a8b9e953-b751-4991-b90e-e3ec251f65f0.FOP 1" to: "88a4d6d0-fcf2-4c03-be5e-e6a23c727bf4.FIP 1" }
    functional_exchange "Displayed Passenger Services List" { id: "cf35086b-7e31-4002-ad31-abca2fe14922" from: "83b4863a-7795-4911-aeef-2938443cce3d.FOP1" to: "a3605148-c80e-4635-9790-f068451a1b4a.FIP1" }
    functional_exchange "Displayed Available Languages" { id: "210a7607-bb1a-4a05-9e47-dbc8e977ffe0" from: "c7edaab3-11bc-4409-b300-13c4b4192742.FOP 2" to: "643033c9-5624-45f8-a1ce-b9bc4ddc2553.FIP 1" category: "Displayed VOD Movie Data" }
    functional_exchange "Displayed Movie Description" { id: "1c787f88-e2a9-44a1-94ac-7dde8e02d64e" from: "c7edaab3-11bc-4409-b300-13c4b4192742.FOP 1" to: "643033c9-5624-45f8-a1ce-b9bc4ddc2553.FIP 3" category: "Displayed VOD Movie Data" }
    functional_exchange "VOD Movie Stop Command" { id: "098c7a52-a4f7-4b9a-b0ec-9c3b02b7687e" from: "0c8009a9-0524-4445-a9c8-f0295467f62a.FOP 7" to: "ac5be58c-aa92-4247-8c8f-9867ddbd6289.FIP 2" category: "VOD Movie Control Commands" }
    functional_exchange "Media Request" { id: "24ebc544-379b-4899-a652-6e4ad07835c4" from: "d53e6dd4-3926-4b25-80d1-14553a472f9e.FOP 5" to: "0378eba1-fb7b-4b4d-b39d-7a14ce4e5f4e.FIP 1" exchange_items: ["32703aff-a875-41e8-a93c-20e8a4b8df4b"] category: "Media Requests" }
    functional_exchange "Media Content Request" { id: "b3f119eb-f8a5-47c4-8737-bd2ad6f81b84" from: "1cda3b05-ee17-4912-b875-40bc4ddca86b.FOP 2" to: "0378eba1-fb7b-4b4d-b39d-7a14ce4e5f4e.FIP 2" category: "Media Requests" }
    functional_exchange "Imposed Video Data" { id: "bfa5d01d-8470-4c88-876f-a35f2f749e1f" from: "0378eba1-fb7b-4b4d-b39d-7a14ce4e5f4e.FOP 4" to: "1cda3b05-ee17-4912-b875-40bc4ddca86b.FIP 2" category: "Media Content" }
    functional_exchange "Media Content Request (2)" { id: "a9c000e7-3c6d-45e4-a69c-c774cd5f9af5" from: "dcbf11ee-150a-4afa-8e06-3f0ab9aee27a.FOP 5" to: "0378eba1-fb7b-4b4d-b39d-7a14ce4e5f4e.FIP 3" category: "Media Requests" }
    functional_exchange "VOD Movies List" { id: "b9f1751a-2fe1-4abc-b02c-775b6971ddd4" from: "0378eba1-fb7b-4b4d-b39d-7a14ce4e5f4e.FOP 2" to: "dcbf11ee-150a-4afa-8e06-3f0ab9aee27a.FIP 4" category: "Media Content" }
    functional_exchange "VOD Movie Data" { id: "fffd52c1-8a54-4c09-8929-ce60715f52cc" from: "0378eba1-fb7b-4b4d-b39d-7a14ce4e5f4e.FOP 3" to: "dcbf11ee-150a-4afa-8e06-3f0ab9aee27a.FIP 5" category: "Media Content" }
    functional_exchange "Passenger VOD Service Activation" { id: "59d829d5-b11e-4680-be41-a84d249fe470" from: "8cc1e6f5-dbe2-4629-8a43-b60eef771a8f.FOP 4" to: "241dcfae-1880-45f9-bf2c-32b0cbe736b7.FIP 2" category: "Service Activation / Interruption" }
    functional_exchange "Displayed Imposed Videos List" { id: "b43248e4-c661-45c7-87d7-ec367925b85e" from: "cebc5976-81d6-4718-b1f0-9f8b7687ceeb.FOP1" to: "292159f9-5917-4c0c-9ea3-2800431b3c48.FIP 2" category: "Displayed Imposed Video Data" }
    functional_exchange "VOD Movie Language Selection Command" { id: "1dd52651-c4ce-4253-94ee-2ab5daa2937d" from: "0c8009a9-0524-4445-a9c8-f0295467f62a.FOP 1" to: "dcbf11ee-150a-4afa-8e06-3f0ab9aee27a.FIP 6" }
    functional_exchange "VOD Movie Selection Command" { id: "e13c6b16-a404-43be-9abf-5a7fb7d15dc7" from: "0c8009a9-0524-4445-a9c8-f0295467f62a.FOP 2" to: "dcbf11ee-150a-4afa-8e06-3f0ab9aee27a.FIP 7" }
    functional_exchange "Digital Media (2)" { id: "2fa3bc43-7d00-4b51-ad50-38854356e0d0" from: "eab9aed9-2576-4d7b-ad86-021e79aac4e1.FOP 1" to: "16904687-4018-46cc-a0ec-c415f6d2bc1d.FIP 1" }
    functional_exchange "IFE Operating Profile" { id: "f5307d6d-b27f-4f2d-8817-13ee254f0011" from: "c13d6970-0ca4-44b4-be4b-64b6f3eebbb4.FOP 1" to: "88a4d6d0-fcf2-4c03-be5e-e6a23c727bf4.FIP 2" }
    functional_exchange "Decompression Notification" { id: "20bd2597-5ddc-4c0d-8f77-2acdf1e96712" from: "b4c36894-10af-4166-9da7-099de70ad0f6.FOP 1" to: "c13d6970-0ca4-44b4-be4b-64b6f3eebbb4.FIP 1" }
    functional_exchange "Electrical Power Profile" { id: "37feeb44-e244-49db-b064-9e2e33f44f3d" from: "b4c36894-10af-4166-9da7-099de70ad0f6.FOP 2" to: "c13d6970-0ca4-44b4-be4b-64b6f3eebbb4.FIP 2" }
    functional_exchange "Displayed Video" { id: "4a0655f4-d7ed-407c-80e1-e5fc564d893c" from: "bd383ff2-258a-41dd-973f-07f7aa30da7a.FOP 2" to: "649d218e-6119-46b3-8c9e-377cf40e086d.FIP2" }
    functional_exchange "Audio Announcement" { id: "249a8a73-3da3-46c3-92e4-b88f0114dcf3" from: "820d4407-7206-4075-91fc-d76ce083056e.FOP 5" to: "2788898f-6daf-43c1-bca8-6ab484ac24c6.FIP 1" }
    functional_exchange "Audio Signal (2)" { id: "e4f320ab-7513-45d9-a259-2f035067664c" from: "9c247f2d-ed9b-43e8-921a-d1fdbb331ab1.FOP1" to: "3159e079-92f0-4e22-8cef-53e8f447cd5f.FIP 1" }
    functional_exchange "VOD Movie Stop Selection" { id: "4610b9a6-ecd4-4f0d-880a-4d3a81793f1c" from: "643033c9-5624-45f8-a1ce-b9bc4ddc2553.FOP 1" to: "0c8009a9-0524-4445-a9c8-f0295467f62a.FIP 1" category: "VOD Movie Control Commands" }
    functional_exchange "VOD Movie Play Selection" { id: "8cce0d07-4c29-4816-90db-8e4cf51287ad" from: "643033c9-5624-45f8-a1ce-b9bc4ddc2553.FOP 2" to: "0c8009a9-0524-4445-a9c8-f0295467f62a.FIP 2" category: "VOD Movie Control Commands" }
    functional_exchange "VOD Movie Pause Selection" { id: "21fb7730-03d0-44c7-ac6d-8f532c57a3da" from: "643033c9-5624-45f8-a1ce-b9bc4ddc2553.FOP 3" to: "0c8009a9-0524-4445-a9c8-f0295467f62a.FIP 3" category: "VOD Movie Control Commands" }
    functional_exchange "VOD Movie Language Selection" { id: "fb38d44c-4891-4a0a-b1ed-f7215109c02f" from: "643033c9-5624-45f8-a1ce-b9bc4ddc2553.FOP 4" to: "0c8009a9-0524-4445-a9c8-f0295467f62a.FIP 4" category: "VOD Selections" }
    functional_exchange "VOD Movie Selection" { id: "00c1bc21-a090-473b-8a41-af5987bfe9b5" from: "643033c9-5624-45f8-a1ce-b9bc4ddc2553.FOP 5" to: "0c8009a9-0524-4445-a9c8-f0295467f62a.FIP 5" category: "VOD Selections" }
    functional_exchange "VOD Movie Resume Command" { id: "ee003445-2cc3-47b3-a7b1-2891d7eb8ca1" from: "241dcfae-1880-45f9-bf2c-32b0cbe736b7.FOP 1" to: "ac5be58c-aa92-4247-8c8f-9867ddbd6289.FIP 1" }
    functional_exchange "Displayed Imposed Video Playing Status" { id: "27dd06a2-8e72-4694-9fbe-f168f0bc8198" from: "72f0a4da-866e-4136-8ca7-bfc85db05005.FOP 1" to: "292159f9-5917-4c0c-9ea3-2800431b3c48.FIP 3" }
    functional_exchange "Displayed Audio Interruption Screen" { id: "e3a13e8a-9c88-4a6a-82bc-03f0c2ac8bbd" from: "068dc2db-3e29-421d-84be-8444ce8acc9c.FOP 1" to: "3159e079-92f0-4e22-8cef-53e8f447cd5f.FIP 2" }
    functional_exchange "Electrical Power Profile (2)" { id: "fa61847f-3abe-4ab2-a857-72bd5e6b4d01" from: "4c237128-0cf3-4074-8af4-5e63e4994e02.FOP 1" to: "b4c36894-10af-4166-9da7-099de70ad0f6.FIP 1" }
    functional_exchange "Decompression Notificaton" { id: "fabe2bde-f0f1-4275-a0b0-82bf22e55819" from: "75e214e4-74f8-4c44-97f9-f328b8ac3f97.FOP 1" to: "b4c36894-10af-4166-9da7-099de70ad0f6.FIP 2" }
    functional_exchange "Imposed Video Play Selection" { id: "7c8c6379-58d3-45af-b57f-e571c28aade9" from: "292159f9-5917-4c0c-9ea3-2800431b3c48.FOP 1" to: "9b798c7d-2d5a-43f9-867d-b26c3f4c1337.FIP 1" categories: ["Imposed Video Control Commands", "Secondary Imposed Video Control Commands"] }
    functional_exchange "Imposed Video Pause Selection" { id: "70bdc1ce-928c-4117-98c0-2094a46701bb" from: "292159f9-5917-4c0c-9ea3-2800431b3c48.FOP 2" to: "9b798c7d-2d5a-43f9-867d-b26c3f4c1337.FIP 2" categories: ["Imposed Video Control Commands", "Secondary Imposed Video Control Commands"] }
    functional_exchange "Imposed Video Stop Selection" { id: "42c63e90-bab0-45ba-b28f-e7384174f179" from: "292159f9-5917-4c0c-9ea3-2800431b3c48.FOP 3" to: "9b798c7d-2d5a-43f9-867d-b26c3f4c1337.FIP 3" categories: ["Imposed Video Control Commands", "Secondary Imposed Video Control Commands"] }
    functional_exchange "Imposed Video List" { id: "4d63300c-14d0-4d28-9cd1-af3a6666088a" from: "0378eba1-fb7b-4b4d-b39d-7a14ce4e5f4e.FOP 5" to: "1cda3b05-ee17-4912-b875-40bc4ddca86b.FIP 3" }
    functional_exchange "Imposed Video Selection" { id: "773ef288-a678-43c7-93a1-e48c0aa06593" from: "292159f9-5917-4c0c-9ea3-2800431b3c48.FOP 4" to: "9b798c7d-2d5a-43f9-867d-b26c3f4c1337.FIP 4" }
    functional_exchange "Imposed Video Play Command" { id: "0a2779ea-7dd6-4f2c-96d9-c9b18788dbd4" from: "9b798c7d-2d5a-43f9-867d-b26c3f4c1337.FOP 3" to: "737aebcf-1f00-40ac-8e03-df8bc6596159.FIP 2" category: "Imposed Video Control Commands" }
    functional_exchange "Imposed Video Pause Command" { id: "d6c15a62-01ff-42ca-ad7b-65a726119365" from: "9b798c7d-2d5a-43f9-867d-b26c3f4c1337.FOP 4" to: "737aebcf-1f00-40ac-8e03-df8bc6596159.FIP 3" category: "Imposed Video Control Commands" }
    functional_exchange "Imposed Video Stop Command" { id: "9c056aec-462c-4a65-b7a2-0de60056fae6" from: "9b798c7d-2d5a-43f9-867d-b26c3f4c1337.FOP 5" to: "737aebcf-1f00-40ac-8e03-df8bc6596159.FIP 4" category: "Imposed Video Control Commands" }
    functional_exchange "Audio Signal (3)" { id: "8f4ee259-aa87-4f0e-b867-56fedf4bac7e" from: "a84dff3b-2158-41ed-b96c-0b8ace0bea3b.FOP 1" to: "3159e079-92f0-4e22-8cef-53e8f447cd5f.FIP 3" }
    functional_exchange "Displayed Video (2)" { id: "e06630d1-7431-4730-8c5e-a1ebeef0a9e6" from: "23b2867d-b9cd-4b1d-8d40-7de838fc1313.FOP 1" to: "649d218e-6119-46b3-8c9e-377cf40e086d.FIP 2" }
    functional_exchange "Audio Stream (2)" { id: "cd8c1d42-d6a0-413b-b741-ff42f3d6a375" from: "ccf41abe-2a9d-4f50-bc3d-643397b7a338.FOP 1" to: "9c247f2d-ed9b-43e8-921a-d1fdbb331ab1.FIP 1" }
    physical_link "Gigabit Ethernet" { id: "40849d2f-1ebc-4bde-8d83-065004bdc7f5" from: "3e6b5b54-0616-4c9e-a017-86f574e2e98f.PP 2" to: "ee7ecd3f-bf35-4657-b7bd-1e45a8fc025c.PP 4" }
    physical_link "Gigabit Ethernet (2)" { id: "c4de8d15-7a19-4de0-aad6-0ec1b74be12d" from: "ee7ecd3f-bf35-4657-b7bd-1e45a8fc025c.PP2" to: "22cbea03-a250-4ea8-a6b3-63829196feb1.PP 1" }
    physical_link "USB" { id: "cc84a961-0450-4dd1-870b-5e791f9b68d4" from: "22cbea03-a250-4ea8-a6b3-63829196feb1.PP 2" to: "5ed18efd-1535-4aa3-aaaf-48a3ca867607.PP1" }
    physical_link "USB (2)" { id: "e901bc3c-c5ee-4d6d-b608-60daba0a9f95" from: "5f666fd8-8032-45f7-821f-1bccbe117b50.PP1" to: "a850426c-c425-4c85-85fa-cd85b0c066e5.PP2" }
    physical_link "Ethernet" { id: "a9ee30dc-5371-4d3d-ba41-ade9e32298ae" from: "01cf8dac-9d35-43aa-a717-5e740989fdc1.PP 4" to: "ee7ecd3f-bf35-4657-b7bd-1e45a8fc025c.PP 5" }
    physical_link "HDMI Cable" { id: "3298fca7-7b1b-49de-89e6-002545b6da68" from: "3e6b5b54-0616-4c9e-a017-86f574e2e98f.PP 3" to: "404eeafa-1b20-4162-893c-f0cfc1e836d6.PP 1" }
    physical_link "HDMI Cable (2)" { id: "33c2368e-47ba-498b-8458-04417468cb20" from: "404eeafa-1b20-4162-893c-f0cfc1e836d6.PP 2" to: "dcb6cca2-ea15-46f9-824f-530d40b4d5f9.PP 1" }
    physical_link "Gigabit Ethernet (3)" { id: "b897b8a1-0131-4e85-8ee3-fd7d981dc583" from: "a850426c-c425-4c85-85fa-cd85b0c066e5.PP 4" to: "ee7ecd3f-bf35-4657-b7bd-1e45a8fc025c.PP 7" }
    physical_link "SATA" { id: "4c43a18c-8a93-49d0-87d6-606e8b8e5e83" from: "c1cbb014-c2cd-403b-a9c1-62f438e79c8c.PP2" to: "0cb2ede5-9ff4-417c-b6c4-c4e4f100ebed.PP1" }
    physical_link "Ethernet Connector" { id: "7666deb9-20e2-4545-a1e7-f95d979c7522" from: "c1cbb014-c2cd-403b-a9c1-62f438e79c8c.PP 2" to: "a850426c-c425-4c85-85fa-cd85b0c066e5.PP 4" }
    physical_link "HDMI" { id: "51d12068-0fc1-4311-a3b4-8cdfcc4f8f5b" from: "44003100-30d6-4894-af55-e2e982a5b921.PP2" to: "6d095cde-706f-41c2-91ab-7c5189f358c4.PP3" }
    physical_link "Ethernet Connector (2)" { id: "17d8f2f2-b529-483c-a1a6-336476ca7d6e" from: "22cbea03-a250-4ea8-a6b3-63829196feb1.PP 1" to: "6d095cde-706f-41c2-91ab-7c5189f358c4.PP 4" }
    physical_link "BUS connector" { id: "c9acf47f-f9b3-47f4-a405-46e7a9c31438" from: "6d095cde-706f-41c2-91ab-7c5189f358c4.PP 5" to: "2632079f-6a54-406a-a957-011c53b16209.PP 1" }
    physical_link "BUS connector (2)" { id: "256327ae-261b-44f4-83e3-5c1de96928da" from: "78c650a1-7440-4335-bf95-f421abdf2c0f.PP 1" to: "2632079f-6a54-406a-a957-011c53b16209.PP 2" }
    physical_link "BUS connector (3)" { id: "c5c72d64-7293-4079-be51-a8c8baa99f17" from: "3d96ae05-f435-49b3-abd1-574551f38704.PP 2" to: "2632079f-6a54-406a-a957-011c53b16209.PP 3" }
    physical_link "BUS connector (4)" { id: "c25bf1c6-e228-46c3-9998-bf217dd457dc" from: "e93cee23-c12e-43e4-ac6d-6049aeb13a63.PP 1" to: "2632079f-6a54-406a-a957-011c53b16209.PP 4" }
    physical_link "USB (3)" { id: "d940e7f2-9ff1-4e9b-b762-4fd0ba452b6a" from: "22cbea03-a250-4ea8-a6b3-63829196feb1.PP 2" to: "6d095cde-706f-41c2-91ab-7c5189f358c4.PP 6" }
    physical_link "Internal BUS" { id: "192653bc-f49c-4c9e-845b-7568a69b96fa" from: "fec9b0d7-7a0b-4386-b1e3-4cccb2c721c3.PP 1" to: "f52035f8-64d3-4259-8aef-5ad19157efca.PP 1" }
    physical_link "Ethernet Connector (3)" { id: "3730671a-8242-4329-9843-b663f3a4cadb" from: "fec9b0d7-7a0b-4386-b1e3-4cccb2c721c3.PP 2" to: "01cf8dac-9d35-43aa-a717-5e740989fdc1.PP 4" }
    physical_link "Ethernet Connector (4)" { id: "e4ea34bf-b171-4569-b305-2cf91d839a45" from: "3e6b5b54-0616-4c9e-a017-86f574e2e98f.PP 2" to: "820e9f56-cac0-4043-ba05-19763e6e3788.PP 1" }
    physical_link "HDMI Connector" { id: "1f390637-4609-4ec8-9dee-65f7e9fd3a41" from: "820e9f56-cac0-4043-ba05-19763e6e3788.PP 2" to: "3e6b5b54-0616-4c9e-a017-86f574e2e98f.PP 3" }
    physical_exchange "Passenger Selections" { id: "75e86175-6b6a-4fba-a9d4-9089fcbd80db" from: "ca1519c5-e748-43ee-a81b-474a31b7b5ab.CP 1" to: "9eef8f0f-1bf8-4e51-a17f-480b3afbfd85.CP 1" }
    physical_exchange "Displayed Elements" { id: "78a63be4-49c7-48ba-9dc5-5492f0237cca" from: "9eef8f0f-1bf8-4e51-a17f-480b3afbfd85.CP 2" to: "ca1519c5-e748-43ee-a81b-474a31b7b5ab.CP 2" }
    physical_exchange "Displayed Elements (2)" { id: "cc675516-b935-4728-a193-216e720a454b" from: "abc69d28-d5c4-470e-afd1-89daa1cc81d2.CP 1" to: "a5782ec2-f546-49e5-98aa-88146d997d1d.CP 1" }
    physical_exchange "Selections" { id: "0bcc0df3-a5b8-4a75-a0bd-423d9dfcbe8b" from: "a5782ec2-f546-49e5-98aa-88146d997d1d.CP 2" to: "abc69d28-d5c4-470e-afd1-89daa1cc81d2.CP 2" }
    physical_exchange "Displayed Elements (3)" { id: "a912b5c2-30ad-4f8d-b61a-c2e0950ab7cd" from: "0c7d06de-bc6a-44d2-8b01-459cbdbbf69a.CP 1" to: "ca1519c5-e748-43ee-a81b-474a31b7b5ab.CP 3" }
    physical_exchange "Media Data Requests" { id: "e501590f-c85d-4cad-9a7c-1ecce7015320" from: "a6815c30-8c2b-4273-bbf1-450bb1df58ed.CP 4" to: "eee15697-a375-4a9d-ba93-7ec5b34bc63c.CP 1" }
    physical_exchange "Media Content" { id: "c27d12d2-6121-4e69-9342-bf5126ebca24" from: "eee15697-a375-4a9d-ba93-7ec5b34bc63c.CP 2" to: "a6815c30-8c2b-4273-bbf1-450bb1df58ed.CP 5" }
    physical_exchange "Media Data Requests (2)" { id: "82c4cf1f-6118-4d53-98e5-933f86d9b04c" from: "647781a2-de41-4fa1-a6e6-3a886a706c4e.CP 4" to: "eee15697-a375-4a9d-ba93-7ec5b34bc63c.CP 3" }
    physical_exchange "Media Content (2)" { id: "ab2fbea9-7c9d-47c5-adf1-43820589478f" from: "eee15697-a375-4a9d-ba93-7ec5b34bc63c.CP 4" to: "647781a2-de41-4fa1-a6e6-3a886a706c4e.CP 5" }
    physical_exchange "Streaming Protocol" { id: "282d427b-d3c9-4eb9-973c-4d45ed593234" from: "3f745bea-11ae-4b8c-8a06-930f2391565e.CP 1" to: "819adf3f-da1d-4d2b-8faf-3b1d7b99503d.CP 1" }
    physical_exchange "Packet Transmission" { id: "514e880e-2ecf-4f3e-a88c-2f3ced8ca714" from: "819adf3f-da1d-4d2b-8faf-3b1d7b99503d.CP 2" to: "34420358-2236-4010-8504-5fab9af55c0d.CP 1" }
    physical_path "PVDU BUS" {
        id: "7311aa65-e4bf-4e93-a493-e6bb442ee6a0"
        involves: ["BUS connector", "BUS connector (3)", "BUS connector (2)", "BUS connector (4)"]
    }
    physical_path "Network Path" {
        id: "fdc63ce0-6f1d-44f6-942e-8d414a8df2e7"
        involves: ["Ethernet Connector", "Gigabit Ethernet (3)", "Gigabit Ethernet (2)", "Ethernet Connector (2)"]
    }
    functional_chain "PFC AND" {
        id: "32b16a4d-6319-45c7-be3c-fb276e324a19"
        owner: "f46f4ddc-51cc-4b2c-ba6c-d9e8c0bae8ea"
        involves: ["88688938-d911-4317-b36e-266cef11a7f3", "fd6de739-ae93-4e5f-acaf-4e8fb8f0b2e5", "1c49d7b2-bb7a-4939-86c1-a73c0822fdf5", "0012d480-7be3-452f-9872-3a0d8868e2f6", "79218945-69c6-4bc6-8aa6-9d67f537492d"]
        control and "AND 1" { id: "f99cae18-4781-46b0-b6c1-307c24b0679e" }
        control and "AND 2" { id: "800e54fc-f4d1-438d-ac58-2321d92c7b17" }
        sequence "fd6de739-ae93-4e5f-acaf-4e8fb8f0b2e5" -> "AND 1" { id: "8784bebf-d0a9-4952-bc51-00085d10d15d" }
        sequence "AND 1" -> "79218945-69c6-4bc6-8aa6-9d67f537492d" { id: "e88f1849-7afe-41f1-b4aa-da09f06d8d2a" condition: "Condition A" }
        sequence "AND 1" -> "1c49d7b2-bb7a-4939-86c1-a73c0822fdf5" { id: "f6a819bd-1ab6-492c-b104-cffe5d26e7cd" condition: "Condition B" }
        sequence "79218945-69c6-4bc6-8aa6-9d67f537492d" -> "AND 2" { id: "e0b677ed-fc9e-43ec-8eea-1142f8074bc8" }
        sequence "AND 2" -> "0012d480-7be3-452f-9872-3a0d8868e2f6" { id: "029ffad3-e8ab-4af6-8019-213751ef0adb" }
        sequence "1c49d7b2-bb7a-4939-86c1-a73c0822fdf5" -> "AND 2" { id: "8a7d0ce0-9cab-465c-81ba-141398e66e86" }
    }
    functional_chain "PFC IT" {
        id: "05185f6f-bb2c-489e-8492-70c6af579979"
        owner: "f46f4ddc-51cc-4b2c-ba6c-d9e8c0bae8ea"
        involves: ["0012d480-7be3-452f-9872-3a0d8868e2f6", "1291c3c4-170d-4fcc-9b59-5b0961cd4a2d", "71147664-0fcd-401f-937a-bdc76216924d", "62f152bd-6017-4828-9034-7af3c387f83a"]
        control iterate "ITERATE 1" { id: "3ed2161f-c41d-4298-8f35-3cf3391e4461" }
        control iterate "ITERATE 2" { id: "58d866fa-463c-419a-ac8a-2216811bc8fe" }
        sequence "0012d480-7be3-452f-9872-3a0d8868e2f6" -> "ITERATE 2" { id: "b041375f-3b7c-4645-8678-afc6dc03977d" }
        sequence "ITERATE 2" -> "1291c3c4-170d-4fcc-9b59-5b0961cd4a2d" { id: "7723d293-3d2d-48c4-ba64-8a37ad655267" }
        sequence "1291c3c4-170d-4fcc-9b59-5b0961cd4a2d" -> "71147664-0fcd-401f-937a-bdc76216924d" { id: "6d90c275-c55f-4a01-93bb-e9f6e9480b48" }
        sequence "71147664-0fcd-401f-937a-bdc76216924d" -> "ITERATE 1" { id: "5fe52982-0b3c-4aea-86e4-ac52113c8b39" }
        sequence "ITERATE 1" -> "62f152bd-6017-4828-9034-7af3c387f83a" { id: "9228b9a7-b1c0-4f86-be52-fabde26e79bc" }
        sequence "ITERATE 1" -> "ITERATE 2" { id: "c44045f9-fc60-44c2-8e8d-26cc2a78334f" }
    }
    functional_chain "PFC of PFCs" {
        id: "9a1fdc9c-09e1-497d-ab3d-3947ba97ea20"
        owner: "f46f4ddc-51cc-4b2c-ba6c-d9e8c0bae8ea"
        involves: ["32b16a4d-6319-45c7-be3c-fb276e324a19", "05185f6f-bb2c-489e-8492-70c6af579979", "1f19f470-fc3b-4bae-9f69-35f96bafa17d"]
    }
    functional_chain "PFC1" {
        id: "1f19f470-fc3b-4bae-9f69-35f96bafa17d"
        owner: "f46f4ddc-51cc-4b2c-ba6c-d9e8c0bae8ea"
        involves: ["29c4014e-7db7-4660-9f3e-e0a8a37e0090", "52d67cae-5ad2-4c5d-b4bf-e68d308c73b3"]
    }
    functional_chain "Watch Imposed Video on Cabin Screen" {
        id: "fc974a03-0fc7-48df-aae6-80156f9920a8"
        owner: "7f111133-ec10-4aab-ab54-936a20776bd2"
        involves: ["292159f9-5917-4c0c-9ea3-2800431b3c48", "9b798c7d-2d5a-43f9-867d-b26c3f4c1337", "737aebcf-1f00-40ac-8e03-df8bc6596159", "d53e6dd4-3926-4b25-80d1-14553a472f9e", "0378eba1-fb7b-4b4d-b39d-7a14ce4e5f4e", "d53e6dd4-3926-4b25-80d1-14553a472f9e", "e8af73d3-bacb-4222-b387-f6b0bd4d13a1", "72f0a4da-866e-4136-8ca7-bfc85db05005", "292159f9-5917-4c0c-9ea3-2800431b3c48", "98960be5-3e8f-4743-ab70-805645472512", "2dfb0a52-7568-4903-9c78-1e462848f868", "28b23885-f94c-44b9-8c2f-a2f52b379842", "c91204f5-d5ed-401e-b83d-d6e85d7d887c", "23b2867d-b9cd-4b1d-8d40-7de838fc1313", "649d218e-6119-46b3-8c9e-377cf40e086d", "ccf41abe-2a9d-4f50-bc3d-643397b7a338", "9c247f2d-ed9b-43e8-921a-d1fdbb331ab1"]
    }
    functional_chain "Watch Imposed Video on Private Screen" {
        id: "0a4c9c3f-cd72-4d96-b87b-e2be6e3a9b5f"
        owner: "7f111133-ec10-4aab-ab54-936a20776bd2"
        involves: ["292159f9-5917-4c0c-9ea3-2800431b3c48", "9b798c7d-2d5a-43f9-867d-b26c3f4c1337", "737aebcf-1f00-40ac-8e03-df8bc6596159", "3f69f8bd-704d-438b-8f05-fc7d56363811", "241dcfae-1880-45f9-bf2c-32b0cbe736b7", "8daae3ab-944b-4ade-9896-0d51c62ef3e8", "d53e6dd4-3926-4b25-80d1-14553a472f9e", "0378eba1-fb7b-4b4d-b39d-7a14ce4e5f4e", "d53e6dd4-3926-4b25-80d1-14553a472f9e", "e8af73d3-bacb-4222-b387-f6b0bd4d13a1", "72f0a4da-866e-4136-8ca7-bfc85db05005", "292159f9-5917-4c0c-9ea3-2800431b3c48", "98960be5-3e8f-4743-ab70-805645472512", "2dfb0a52-7568-4903-9c78-1e462848f868", "4c361e00-d6d0-4acf-a523-1de11acf7797", "ecc7d466-f9cb-4be1-a099-b8449146cf18", "9d3a7965-d967-42e2-8f25-c0113abaf9ff", "bd383ff2-258a-41dd-973f-07f7aa30da7a", "be579285-807c-46ab-bcab-ae7efb19a8d6"]
    }
    functional_chain "Broadcast Audio Announcement" {
        id: "c4a51bed-c9eb-4784-8b30-f3fa4e19a51c"
        owner: "7f111133-ec10-4aab-ab54-936a20776bd2"
        involves: ["820d4407-7206-4075-91fc-d76ce083056e", "2788898f-6daf-43c1-bca8-6ab484ac24c6", "80a8d0f6-ca9e-486f-9ac3-0c524de61281", "9155e7ae-b4c3-4f06-b8b2-0ec90045ac8e", "07823fa4-38c9-43fe-8b4b-8aa5588b7c46", "a84dff3b-2158-41ed-b96c-0b8ace0bea3b", "3159e079-92f0-4e22-8cef-53e8f447cd5f", "3f69f8bd-704d-438b-8f05-fc7d56363811", "068dc2db-3e29-421d-84be-8444ce8acc9c", "241dcfae-1880-45f9-bf2c-32b0cbe736b7", "8daae3ab-944b-4ade-9896-0d51c62ef3e8"]
    }
    functional_chain "Stop Playing Imposed Video [NOT DONE]" {
        id: "a4edf6d8-4143-499c-9e56-57c95d970304"
        owner: "7f111133-ec10-4aab-ab54-936a20776bd2"
    }
    functional_chain "Pause Running Imposed Video [NOT DONE]" {
        id: "50f542b7-d31f-44cc-8cf6-9a153d604ea7"
        owner: "7f111133-ec10-4aab-ab54-936a20776bd2"
    }
    functional_chain "Start Playing VOD Movie" {
        id: "6b2127cf-5024-4ab1-a1ce-cd743982ccde"
        owner: "6de68210-1c70-4531-9b35-4f4a8c959220"
        involves: ["080f330d-2364-4093-8f09-ffecfe2fffd0", "643033c9-5624-45f8-a1ce-b9bc4ddc2553", "0c8009a9-0524-4445-a9c8-f0295467f62a", "ac5be58c-aa92-4247-8c8f-9867ddbd6289", "d53e6dd4-3926-4b25-80d1-14553a472f9e", "0378eba1-fb7b-4b4d-b39d-7a14ce4e5f4e", "d53e6dd4-3926-4b25-80d1-14553a472f9e"]
    }
    functional_chain "Resume VOD Movie" {
        id: "f53e78f2-08f5-437f-964b-583a12ecf181"
        owner: "6de68210-1c70-4531-9b35-4f4a8c959220"
        involves: ["080f330d-2364-4093-8f09-ffecfe2fffd0", "8daae3ab-944b-4ade-9896-0d51c62ef3e8", "241dcfae-1880-45f9-bf2c-32b0cbe736b7", "ac5be58c-aa92-4247-8c8f-9867ddbd6289", "d53e6dd4-3926-4b25-80d1-14553a472f9e", "0378eba1-fb7b-4b4d-b39d-7a14ce4e5f4e", "d53e6dd4-3926-4b25-80d1-14553a472f9e"]
    }
    functional_chain "Broadcast VOD Movie" {
        id: "080f330d-2364-4093-8f09-ffecfe2fffd0"
        owner: "6de68210-1c70-4531-9b35-4f4a8c959220"
        involves: ["98960be5-3e8f-4743-ab70-805645472512", "2dfb0a52-7568-4903-9c78-1e462848f868", "4c361e00-d6d0-4acf-a523-1de11acf7797", "ecc7d466-f9cb-4be1-a099-b8449146cf18", "9d3a7965-d967-42e2-8f25-c0113abaf9ff", "bd383ff2-258a-41dd-973f-07f7aa30da7a", "be579285-807c-46ab-bcab-ae7efb19a8d6", "9155e7ae-b4c3-4f06-b8b2-0ec90045ac8e", "07823fa4-38c9-43fe-8b4b-8aa5588b7c46", "a84dff3b-2158-41ed-b96c-0b8ace0bea3b", "3159e079-92f0-4e22-8cef-53e8f447cd5f"]
    }
    functional_chain "VOD Service Activation [TODO]" {
        id: "2ad3882f-92d3-4ed2-8fa0-74ec936fcd0d"
        owner: "6de68210-1c70-4531-9b35-4f4a8c959220"
    }
    functional_chain "VOD Movie Interruption [TODO]" {
        id: "6175d22f-cb1d-4112-bdfd-7ef20ecdb8c5"
        owner: "6de68210-1c70-4531-9b35-4f4a8c959220"
    }
    functional_chain "Stop VOD Movie Play [TODO]" {
        id: "cff95325-b878-4954-ab6c-c8207f8f378b"
        owner: "6de68210-1c70-4531-9b35-4f4a8c959220"
    }
    capability_realization "Provide Audio and Video Intercommunication Means" {
        id: "7f111133-ec10-4aab-ab54-936a20776bd2"
        involves: ["8aba6e60-54e0-401a-bc8b-c86090e030d0", "ca1519c5-e748-43ee-a81b-474a31b7b5ab", "a5782ec2-f546-49e5-98aa-88146d997d1d", "0c7d06de-bc6a-44d2-8b01-459cbdbbf69a", "ce7ff26b-6729-420e-b6a7-78fb80634d88", "abc69d28-d5c4-470e-afd1-89daa1cc81d2", "3f745bea-11ae-4b8c-8a06-930f2391565e", "cdc16f53-6eaa-4d74-8d63-73afc483dece", "a6815c30-8c2b-4273-bbf1-450bb1df58ed", "eee15697-a375-4a9d-ba93-7ec5b34bc63c", "3cba132d-d6ec-4839-b597-2183f9f998b2", "3de7180a-c2f0-48ea-9a9e-6f6e6849c651", "a2bf21fb-902c-4b29-87c3-d3436dedb1db", "647781a2-de41-4fa1-a6e6-3a886a706c4e", "d187e156-9b25-4adc-8b97-576853d16e40", "34420358-2236-4010-8504-5fab9af55c0d", "737331c9-23e8-4588-969a-fd15444cafa6", "819adf3f-da1d-4d2b-8faf-3b1d7b99503d", "fc974a03-0fc7-48df-aae6-80156f9920a8", "50f542b7-d31f-44cc-8cf6-9a153d604ea7", "a4edf6d8-4143-499c-9e56-57c95d970304", "c4a51bed-c9eb-4784-8b30-f3fa4e19a51c", "0a4c9c3f-cd72-4d96-b87b-e2be6e3a9b5f"]
        realizes: "e373786f-d962-4a05-9360-902a524b7522"
    }
    capability_realization "Provide Video Entertainment Services" {
        id: "6de68210-1c70-4531-9b35-4f4a8c959220"
        description: "[Wikipedia] Video entertainment is provided via a large video screen at the front of a cabin section, as well as via smaller monitors situated every few rows above the aisles. Sound is supplied via the same headphones as those distributed for audio entertainment. Personal televisions (PTVs) for every passenger provide passengers with channels broadcasting new and classic films, as well as comedies, news, sports programming, documentaries, children's shows, and drama series. Some airlines also present news and current affairs programming, which are often pre-recorded and delivered in the early morning before flights commence. PTVs are operated via an In flight Management System which stores pre-recorded channels on a central server and streams them to PTV equipped seats during flight. VOD (Video On Demand) systems store individual programs separately, allowing a passenger to have a specific program streamed to them privately, and be able to control the playback."
        involves: ["ca1519c5-e748-43ee-a81b-474a31b7b5ab", "eee15697-a375-4a9d-ba93-7ec5b34bc63c", "3f745bea-11ae-4b8c-8a06-930f2391565e", "647781a2-de41-4fa1-a6e6-3a886a706c4e", "819adf3f-da1d-4d2b-8faf-3b1d7b99503d", "ec105112-c640-4de6-bc28-461581a1bace", "34420358-2236-4010-8504-5fab9af55c0d", "d187e156-9b25-4adc-8b97-576853d16e40", "3de7180a-c2f0-48ea-9a9e-6f6e6849c651", "3cba132d-d6ec-4839-b597-2183f9f998b2", "2ad3882f-92d3-4ed2-8fa0-74ec936fcd0d", "6175d22f-cb1d-4112-bdfd-7ef20ecdb8c5", "cff95325-b878-4954-ab6c-c8207f8f378b", "f53e78f2-08f5-437f-964b-583a12ecf181", "6b2127cf-5024-4ab1-a1ce-cd743982ccde", "080f330d-2364-4093-8f09-ffecfe2fffd0"]
        realizes: "58c038e4-77d4-4b0e-9a5d-7f4253285be2"
    }
    capability_realization "Provide Video Gaming Services" {
        id: "c8cff209-41fb-4d61-8cd3-f892a3702355"
        description: "[Wikipedia] Video games are another emerging facet of in-flight entertainment. Some game systems are networked to allow interactive playing by multiple passengers. Later generations of IFE games began to shift focus from pure entertainment to learning. The best examples of this changing trend are the popular trivia game series and the Berlitz Word Traveler that allows passengers to learn a new language in their own language. Appearing as a mixture of lessons and mini games, passengers can learn the basics of a new language while being entertained"
        realizes: "ae98c518-b305-41e6-b825-aa231d354325"
    }
    capability_realization "Provide Wi-Fi Connectivity Service" {
        id: "f6e904de-5549-4e9c-b7ae-aab2fc071e04"
        description: "[Wikipedia] In-flight internet service is provided either through a satellite network or an air-to-ground network. Data communication via satellite system allows passengers to connect to live internet from the individual IFE units or their laptops via the in-flight Wi-Fi access"
        realizes: "24c03d22-ddf6-4525-aa1b-b65e1360fe85"
    }
    capability_realization "Provide Satellite and Internal Telephony Services" {
        id: "2a255ac8-027a-430a-b34f-fc820a3c8478"
        description: "[Wikipedia] Some airlines provide satellite telephones integrated into their system. These are either found at strategic locations in the aircraft or integrated into the passenger remote control used for the individual in-flight entertainment. Passengers can use their credit card to make phone calls anywhere on the ground. These systems are usually not capable of receiving incoming calls. There are also some aircraft that allow faxes to be sent and the rate is usually the same as the call rate, but at a per page rate. Some systems also allow the transmission of SMS. More modern systems allow passengers to call fellow passengers located in another seat by simply keying in the recipient's seat number."
        realizes: "5c8adc1c-079e-4cad-9b33-4dfb53db7f66"
    }
    capability_realization "Provide Moving-Map Services" {
        id: "68ba5d1f-2a3c-4eb5-b19d-d01e2278130f"
        description: "[Wikipedia] A moving-map system is a real-time flight information video channel broadcast through to cabin project/video screens and personal televisions (PTVs). In addition to displaying a map that illustrates the position and direction of the plane, the system gives the altitude, airspeed, outside air temperature, distance to the destination, distance from the origination point, and local time. The moving-map system information is derived in real time from the aircraft's flight computer systems."
        realizes: "9498f50c-65c6-428f-aeb3-a431c5f25e06"
    }
    capability_realization "Provide Audio Entertainment Services" {
        id: "c2e8b925-3dfb-4b87-95c1-5d1ce6266f80"
        description: "[Wikipedia] Audio entertainment covers music, as well as news, information, and comedy. Most music channels are pre-recorded and feature their own DJs to provide chatter, song introductions, and interviews with artists. In addition, there is sometimes a channel devoted to the plane's radio communications, allowing passengers to listen in on the pilot's in-flight conversations with other planes and ground stations."
        realizes: "4c813481-a521-4f4b-904b-7a93569ef620"
    }
    capability_realization "Provide Personal Device Connectivity" {
        id: "37035008-464c-41a6-b270-34eaae1ed7e2"
        realizes: "dfb5ba2c-d16e-4cd6-921f-38a5265c939e"
    }
    capability_realization "Provide Testing Interface" {
        id: "9bd26014-34ac-46c1-ac3c-7646941ada16"
        realizes: "bd2a543a-d87b-4147-8ad4-7f77a110ced2"
    }
    capability_realization "Provide Configuration Means" {
        id: "ca3942df-c48d-461c-9cc2-de207cd35908"
        realizes: "36429bbc-57fd-4862-b59e-003a11f4b146"
    }
}

epbs "EPBS Architecture" {
    id: "7c142d0f-dd8e-4539-bcb5-3ea9c2fdb59d"
    system "System" {
        id: "fb4d3ed0-4de7-43ce-9678-c3d28e3e44f9"
        kind: "SystemCI"
        subsystem "Streaming SW" {
            id: "64961d5d-932d-4c69-8be5-2879940483bc"
            kind: "COTSCI"
            realizes: ["3cba132d-d6ec-4839-b597-2183f9f998b2", "eee15697-a375-4a9d-ba93-7ec5b34bc63c", "e7049747-365c-4c2f-b769-f45221b9a05a", "ce7ff26b-6729-420e-b6a7-78fb80634d88", "3f745bea-11ae-4b8c-8a06-930f2391565e"]
        }
        subsystem "Aircraft Interface" {
            id: "d90f7495-1f1a-4882-95a1-edda9f16fbdf"
            kind: "InterfaceCI"
            realizes: ["cdc16f53-6eaa-4d74-8d63-73afc483dece"]
        }
        subsystem "Seat TV HW" {
            id: "a5d70e56-04c7-4b44-81e4-ce891cae0da3"
            kind: "HWCI"
            realizes: ["22cbea03-a250-4ea8-a6b3-63829196feb1", "5ed18efd-1535-4aa3-aaaf-48a3ca867607"]
        }
        subsystem "Airline-Specific Interaction SW" {
            id: "c241a991-774a-4a82-854a-91d22e804b11"
            kind: "CSCI"
            realizes: ["abc69d28-d5c4-470e-afd1-89daa1cc81d2", "d187e156-9b25-4adc-8b97-576853d16e40", "ec105112-c640-4de6-bc28-461581a1bace"]
        }
        subsystem "Cabin HW" {
            id: "6c0ada8f-a517-4a12-bd8e-12c28ab8b378"
            kind: "HWCI"
            realizes: ["404eeafa-1b20-4162-893c-f0cfc1e836d6", "dcb6cca2-ea15-46f9-824f-530d40b4d5f9"]
        }
        subsystem "Network HW and Cables" {
            id: "5e139f27-c1ea-42d5-a7f0-1cdc6d8bc52c"
            kind: "NDICI"
            realizes: ["33c2368e-47ba-498b-8458-04417468cb20", "ee7ecd3f-bf35-4657-b7bd-1e45a8fc025c", "a9ee30dc-5371-4d3d-ba41-ade9e32298ae", "c4de8d15-7a19-4de0-aad6-0ec1b74be12d", "3298fca7-7b1b-49de-89e6-002545b6da68", "40849d2f-1ebc-4bde-8d83-065004bdc7f5", "b897b8a1-0131-4e85-8ee3-fd7d981dc583"]
        }
        subsystem "Network SW" {
            id: "f6b83d87-9b95-4a3c-aeea-48d755a6eda5"
            kind: "CSCI"
            realizes: ["33ca7fc8-8bd1-48c3-9f22-3fbd217c2549", "809ec42b-c1ee-471f-a45b-d26977dc956a", "403273f6-e4ae-4eed-90f3-d566ecffb01a"]
        }
        subsystem "Core PVDU SW" {
            id: "6e738dc3-2775-4fd8-bec7-376057b88a11"
            kind: "CSCI"
            realizes: ["737331c9-23e8-4588-969a-fd15444cafa6", "647781a2-de41-4fa1-a6e6-3a886a706c4e"]
        }
        subsystem "SystemCI 9" {
            id: "c02105c2-ca0e-44ea-8868-4bf81d6d8e1c"
            kind: "SystemCI"
            realizes: ["a2bf21fb-902c-4b29-87c3-d3436dedb1db", "3e6b5b54-0616-4c9e-a017-86f574e2e98f"]
        }
        subsystem "Video Server HW" {
            id: "79d96a1d-b45a-4621-9c30-3d793fd40ee8"
            kind: "HWCI"
            realizes: ["a850426c-c425-4c85-85fa-cd85b0c066e5"]
        }
    }
}

state_machine "Operational Entity State Machine" {
    owner: "dab8526e-cf3a-4c8a-8fa1-7df4aa281bcf"
    id: "371b47fb-e9a7-4262-aef8-48d4bbc910da"
    initial: "Idle"
    state "Boarding" {
        id: "1bd40278-db4a-4096-a12c-07ed046c5552"
    }
    state "Idle" {
        id: "de7ca93b-c157-4b12-87c7-2af95f01660f"
    }
    state "Takeoff" {
        id: "e3993570-4314-45be-a7c0-bc21b1663bc5"
    }
    state "Cruising" {
        id: "4a069028-2325-48de-b564-4fef9d6d5a24"
    }
    state "Landing" {
        id: "61c11bf7-ea27-40f6-a7a9-361c0482f7f1"
    }
    state "Disembarkation" {
        id: "44b883db-a2bf-4f5e-873b-6e13ca9d9ba6"
    }
    transition "Idle" -> "Boarding" { id: "82717bd3-e9f5-4088-839e-a8e87c24e4ae" trigger: "Boarding Authorization" }
    transition "Boarding" -> "Takeoff" { id: "e7cfe8a6-e1d5-4d74-a84e-18707b887e56" trigger: "Takeolf Authorization" }
    transition "Takeoff" -> "Cruising" { id: "c90681c5-f2de-400d-9d37-a4dec1fbbdc2" trigger: "" events: ["ChangeEvent 1"] }
    transition "Cruising" -> "Landing" { id: "83989ada-854b-4be3-8989-6597b5a3addd" trigger: "Landing Authorization" }
    transition "Landing" -> "Disembarkation" { id: "6d152d53-6e2b-430f-9e43-fa996ec888de" trigger: "Disembarkation Authorization" }
    transition "Disembarkation" -> "Idle" { id: "665632ab-85bf-41ac-ab55-d6dc860be0f0" trigger: "Disembarkation Completed" }
}

state_machine "IFE System / IFE Operating Modes" {
    owner: "e0a46739-7e50-4ea6-b198-0e5efb9ce68f"
    id: "d6609d04-d9ab-4df7-9058-572916f9f3f7"
    initial: "Start Up"
    state "Final" {
        id: "eecdad2c-ece1-44cb-86e1-776d8d80b61f"
        kind: "final"
    }
    mode "Start Up" {
        id: "1656f21f-602e-4a9b-8f53-926f95ff219c"
    }
    mode "Ready" {
        id: "6c60fb24-82ff-482a-87db-58f00e0323f4"
    }
    mode "Fully Operational" {
        id: "6171dcbd-85b7-4452-a035-8ab9c1be37e7"
    }
    mode "Maintenance" {
        id: "751ab990-c2ad-4a61-aa75-ef0ae29f9a6f"
        initial: "Choice"
        state "Choice" {
            id: "b52caea4-8fdb-4bb1-b3c1-0ab7868c2ffe"
            kind: "choice"
        }
        mode "Software Upgrade" {
            id: "890dbd43-d1f3-47fc-98f4-a53f3e2f3c93"
        }
        mode "Content Management" {
            id: "42b447a2-5846-40f8-ad76-2ab9807a85ed"
        }
        mode "Sustaining" {
            id: "0d4ae35a-6c7c-4149-abc7-e3c08efb02b9"
        }
        transition "Choice" -> "Software Upgrade" { id: "42dcb187-c778-4692-b3aa-b771597b8d3d" trigger: "" }
        transition "Choice" -> "Content Management" { id: "5d72f0a9-aab2-48d3-ac82-3ab9cd693594" trigger: "" }
        transition "Choice" -> "Sustaining" { id: "dd08f63d-9af0-48f2-85f5-e79141b43f11" trigger: "" }
    }
    state "Choice (2)" {
        id: "48274f30-48fc-46dc-b4aa-2587972da645"
        kind: "choice"
    }
    mode "Degraded" {
        id: "75503038-7ad6-4212-9098-b5764b7a430b"
    }
    mode "Halted" {
        id: "b2c226dc-43c6-494e-b6cb-dbc652561725"
    }
    transition "Start Up" -> "Ready" { id: "16d80c23-10d1-4a84-af5b-9ed35ab17471" trigger: "" }
    transition "Ready" -> "Choice (2)" { id: "b2f4b8ad-fe74-4318-a088-e7b00f29ed94" trigger: "" }
    transition "Choice (2)" -> "Fully Operational" { id: "2d16e66e-b5e2-4880-ab96-76da319a2103" trigger: "" }
    transition "Choice (2)" -> "Maintenance" { id: "b31765b6-33cc-4df2-b6f0-63fc8fbf5b67" trigger: "" }
    transition "Maintenance" -> "Ready" { id: "487fa6e9-d0dc-422c-8598-4354f290b454" trigger: "" }
    transition "Fully Operational" -> "Ready" { id: "ae0fde45-df0e-4731-b2a0-de26f1d5cb6a" trigger: "" }
    transition "Choice (2)" -> "Final" { id: "ff789a3b-5368-468d-9bae-6f33e1fc77cb" trigger: "" }
    transition "Fully Operational" -> "Degraded" { id: "329d833a-cc85-4d3e-8dad-4a4d14192c57" trigger: "" guard: "Profile = CORE SERVICES ONLY" }
    transition "Degraded" -> "Halted" { id: "10799b35-8076-42f4-8ebc-aca4c16492b5" trigger: "" guard: "Profile = NO SERVICE" }
    transition "Degraded" -> "Final" { id: "533c85d1-0b0f-4c76-8c31-041a87179d12" trigger: "" }
    transition "Halted" -> "Final" { id: "57d28e0e-0696-4a01-b372-c63acc06f75f" trigger: "" }
    transition "Degraded" -> "Fully Operational" { id: "feff77c7-6c73-4525-8f90-7943b89a1d8a" trigger: "" guard: "Profile = ALL SERVICES" }
    transition "Halted" -> "Fully Operational" { id: "689ed0dd-8f30-41a5-b228-0ca8895ccef3" trigger: "" guard: "Profile = ALL SERVICES" }
    transition "Fully Operational" -> "Halted" { id: "83510d8c-3de1-42c3-8bbf-20f0d5cfcdde" trigger: "" guard: "Profile = NO SERVICE" }
}

state_machine "IFE System / PTV Movie Player Modes" {
    owner: "e0a46739-7e50-4ea6-b198-0e5efb9ce68f"
    id: "40074ef0-f899-41bc-8863-c45da1e22d93"
    initial: "Seat TV - Home Page Displayed"
    state "Final" {
        id: "4243ffb3-5f36-4ee3-81bc-d786f8b9424e"
        kind: "final"
    }
    mode "Seat TV - Home Page Displayed" {
        id: "0b71ac24-f73d-4aac-a730-3d6c57ca62bc"
    }
    mode "PTV - Maintenance" {
        id: "2946a0e8-ef0c-4da0-9760-f202d4298f4d"
    }
    mode "Seat TV - End-User Service Running" {
        id: "ed01d361-f9cb-4c26-a0aa-e9338d4066ff"
        mode "Seat TV - VOD Service Activated" {
            id: "49555c2f-b9c9-4ed5-b704-aab0ad79080f"
            initial: "Seat TV - Displaying VOD User Interface"
            mode "Seat TV - VOD Movie Running" {
                id: "13b3665b-619f-4678-8a6f-ccef076b3552"
            }
            mode "Seat TV - Displaying VOD User Interface" {
                id: "ff567a76-0861-45b6-8d64-293758b0dd7f"
            }
            transition "Seat TV - Displaying VOD User Interface" -> "Seat TV - VOD Movie Running" { id: "26009e33-1109-4c48-8df9-44c5a9b6d26f" trigger: "" }
            transition "Seat TV - VOD Movie Running" -> "Seat TV - VOD Movie Paused" { id: "ad2aa322-f575-477d-9bb8-be77f40d5a81" trigger: "" }
            transition "Seat TV - VOD Movie Running" -> "Seat TV - Displaying VOD User Interface" { id: "eaf022e9-435a-4e1c-8da0-a6477f8bc411" trigger: "" }
        }
        mode "Seat TV - News Service Activated" {
            id: "19998744-6825-4911-be1d-e1c973a04511"
        }
        mode "Seat TV - Any Other Service Activated" {
            id: "39e0c808-34e0-4c9e-b82a-85bd2d11b57c"
        }
        mode "Seat TV - Gaming Service Activated" {
            id: "07c746af-a76b-4466-820b-3773214be640"
        }
        mode "Seat TV - Moving-Map Service Activated" {
            id: "e176b21f-d560-4bb2-8fda-f0ec8f9bdb87"
        }
        state "Deep History" {
            id: "bf9bce31-bcc2-43fa-8c87-cd511e2c2c34"
            kind: "deep_history"
        }
        mode "Seat TV - VOD Movie Paused" {
            id: "3c6b34c9-a875-4cc6-9cb6-02b1b4b94432"
        }
        transition "Seat TV - VOD Service Activated" -> "Seat TV - Home Page Displayed" { id: "2f61a941-fb18-4ac9-ade0-22859e7814c7" trigger: "" }
        transition "Seat TV - VOD Movie Paused" -> "Seat TV - VOD Movie Running" { id: "88ef9fad-5981-429d-97c4-ceb49a174c17" trigger: "" }
    }
    mode "Seat TV - Interrupted" {
        id: "e1589350-78e9-43b7-b87d-926def3fdddf"
        mode "Seat TV - Imposed Video Paused" {
            id: "07ab7e94-cc45-4ad3-8a48-ed3f7fed0ec9"
        }
        mode "Seat TV - Imposed Video Running" {
            id: "7104c85d-4fb0-48c6-a16b-2c162fd57a0f"
        }
        mode "Seat TV - Audio Announcement Running" {
            id: "cd6a3e27-e36b-4490-9fca-91784a8fc2dc"
        }
        transition "Seat TV - Imposed Video Running" -> "Seat TV - Imposed Video Paused" { id: "ff455a76-2b69-421b-a315-1ed6a991059d" trigger: "" }
        transition "Seat TV - Imposed Video Paused" -> "Seat TV - Imposed Video Running" { id: "7b4f505b-4cf7-4a06-9fba-cda38f4d515a" trigger: "" }
    }
    transition "Seat TV - Home Page Displayed" -> "Seat TV - Moving-Map Service Activated" { id: "7d63a0c2-2013-4457-9043-6151dceb7afe" trigger: "" }
    transition "Seat TV - Home Page Displayed" -> "Seat TV - News Service Activated" { id: "d6f870b8-c0f4-4c90-994f-0fcda94b2969" trigger: "" }
    transition "Seat TV - Home Page Displayed" -> "Seat TV - Gaming Service Activated" { id: "6c7bfe2c-addc-473d-9381-443727c3f1c2" trigger: "" }
    transition "Seat TV - Home Page Displayed" -> "Seat TV - Any Other Service Activated" { id: "effbd5e4-a19c-4f63-88fa-3c03b0932efa" trigger: "" }
    transition "Seat TV - Home Page Displayed" -> "Final" { id: "2ef387c7-2147-4275-9fe3-d6a3faea5be5" trigger: "" }
    transition "Seat TV - Interrupted" -> "Final" { id: "4596aea0-9ef1-4f03-bf26-d9d61fb259e3" trigger: "" }
    transition "Seat TV - Interrupted" -> "Deep History" { id: "fec67b1b-bd5d-429d-b28a-657c9ee2eb0d" trigger: "" events: ["Imposed Video Ended"] action: ["Handle Service Activation Requests"] }
    transition "Seat TV - End-User Service Running" -> "Final" { id: "76b010f4-8d70-4b39-89ca-cbd2c842e967" trigger: "" }
    transition "Seat TV - Home Page Displayed" -> "PTV - Maintenance" { id: "1c19355c-6b5b-495c-bc09-ff91582a8d56" trigger: "" }
    transition "Seat TV - Home Page Displayed" -> "Seat TV - VOD Service Activated" { id: "4844da87-34dc-4112-8b7d-6f89a76eec35" trigger: "" }
    transition "Seat TV - End-User Service Running" -> "Seat TV - Imposed Video Running" { id: "217b9f27-b9c3-41aa-8c70-4a08cf65938e" trigger: "" }
}

state_machine "IFE System / IFE Operating Modes (2)" {
    owner: "724d6db9-9175-450b-8f05-023fcffec79c"
    id: "3e1b1f3a-e2b0-48fa-b669-7f61f0369e14"
    initial: "Start Up"
    state "Final" {
        id: "00dc870a-b249-493c-8646-4bb995e3a529"
        kind: "final"
    }
    mode "Start Up" {
        id: "fe817497-e397-4053-bda2-4e4684bdcbbd"
    }
    mode "Ready" {
        id: "597598e6-3eaa-4a66-bd87-209a8509b056"
    }
    mode "Fully Operational" {
        id: "39eb29ec-2085-47fe-b3ea-55027fc82faf"
    }
    mode "Maintenance" {
        id: "c7a28614-4584-40b8-90b4-f452d5f458b2"
        initial: "Choice"
        state "Choice" {
            id: "457ecf32-9740-41ee-95e7-dc65266d2b8c"
            kind: "choice"
        }
        mode "Software Upgrade" {
            id: "f59d626b-a356-47dc-8ced-b1803f86427d"
        }
        mode "Content Management" {
            id: "4c9a16e8-ed00-49cd-bab9-177a83db8d86"
        }
        mode "Sustaining" {
            id: "09d7f38a-61ba-4d20-b7f1-a7b6cf3b5353"
        }
        transition "Choice" -> "Software Upgrade" { id: "60761702-8e10-49a4-bb69-afaed788fcd3" trigger: "" }
        transition "Choice" -> "Content Management" { id: "317aed92-040e-4443-830d-7634d0f83cd0" trigger: "" }
        transition "Choice" -> "Sustaining" { id: "104510ea-dbf0-408d-b9cf-d1fba00c31cb" trigger: "" }
    }
    state "Choice (2)" {
        id: "b69efda8-a766-444d-8fab-ecd63db15685"
        kind: "choice"
    }
    mode "Degraded" {
        id: "6d479123-ab40-4f72-a21d-b28be1c0ad54"
    }
    mode "Halted" {
        id: "a07103d1-ed95-45f7-be53-f18fb50dbb1e"
    }
    transition "Start Up" -> "Ready" { id: "e18349e4-0fba-4594-b0bc-cc84bbe957fe" trigger: "" }
    transition "Ready" -> "Choice (2)" { id: "3a658044-9895-49d6-a964-a718147cf1dd" trigger: "" }
    transition "Choice (2)" -> "Fully Operational" { id: "d7fd27a7-1d74-46b1-a702-b5f10faefe33" trigger: "" }
    transition "Choice (2)" -> "Maintenance" { id: "ef3fb471-ce6d-4ffc-9175-ad1d2c84cdb6" trigger: "" }
    transition "Maintenance" -> "Ready" { id: "724a5f01-7425-490d-ae89-37a326651f30" trigger: "" }
    transition "Fully Operational" -> "Ready" { id: "68b7208a-65b0-4a00-939c-89a1f3de3c04" trigger: "" }
    transition "Choice (2)" -> "Final" { id: "aa1e5712-9787-4a92-bfd2-b0e0866ddd3d" trigger: "" }
    transition "Fully Operational" -> "Degraded" { id: "e0da2ce7-c4ef-49de-921c-100d0b130da9" trigger: "" guard: "Profile = CORE SERVICES ONLY" }
    transition "Degraded" -> "Halted" { id: "a16ecc63-804b-47ba-88d3-878a32f0b4dc" trigger: "" guard: "Profile = NO SERVICE" }
    transition "Degraded" -> "Final" { id: "d8678f49-b2f5-446f-bb8e-48c3c735f436" trigger: "" }
    transition "Halted" -> "Final" { id: "23b0b3c6-69e9-46ef-ab7c-91ebc1ea2eb9" trigger: "" }
    transition "Degraded" -> "Fully Operational" { id: "10b05d4a-0c62-400a-9e75-25592b7f5ff3" trigger: "" guard: "Profile = ALL SERVICES" }
    transition "Halted" -> "Fully Operational" { id: "19fa1b59-c961-4897-9c20-87e61b7e91c8" trigger: "" guard: "Profile = ALL SERVICES" }
    transition "Fully Operational" -> "Halted" { id: "a6858931-fedd-4121-a5d6-c14742de4251" trigger: "" guard: "Profile = NO SERVICE" }
}

state_machine "Seat TV / PTV Movie Player Modes" {
    owner: "_8rBvcI2bEd6Q36a6kYkaHQ"
    id: "8b3ccd92-eb7b-4def-8f39-9c02c0bc15fe"
    initial: "Seat TV - Home Page Displayed"
    state "Final" {
        id: "5cddbe8c-c99e-4f3d-8f65-05de353dfa55"
        kind: "final"
    }
    mode "Seat TV - Home Page Displayed" {
        id: "1111bdb8-3546-4266-8d7e-7311c26d2364"
    }
    mode "PTV - Maintenance" {
        id: "c8f57673-e7db-49e2-9850-2a3433e95a89"
    }
    mode "Seat TV - End-User Service Running" {
        id: "ee4a731b-d142-41f2-9a8f-679c5426b7fe"
        mode "Seat TV - VOD Service Activated" {
            id: "bfed98b8-17a1-411b-b5d2-06a282a1a1ce"
            initial: "Seat TV - Displaying VOD User Interface"
            mode "Seat TV - VOD Paused" {
                id: "6107e0ce-8242-4526-b609-b54247d998be"
            }
            mode "Seat TV - VOD Playing" {
                id: "afc7e130-d4ac-4edd-acb2-6ef2fa45c556"
            }
            mode "Seat TV - Displaying VOD User Interface" {
                id: "0a65b216-6ce5-4d16-b4e1-1cfa89dbbe67"
            }
            transition "Seat TV - Displaying VOD User Interface" -> "Seat TV - VOD Playing" { id: "acdf1a44-f0cd-4a0a-ad11-f5f1f35d6756" trigger: "" }
            transition "Seat TV - VOD Paused" -> "Seat TV - VOD Playing" { id: "e89bd0b4-dd20-45ae-aefa-dee8f37a5ed9" trigger: "" guard: "a href=e5bebd43-6881-4c89-be4a-7cc97632df51/.a href=af503302-d4f6-4a93-87b6-e958e9370286/" }
            transition "Seat TV - VOD Playing" -> "Seat TV - VOD Paused" { id: "9eb03518-bde0-4535-a217-8cffbf9ea316" trigger: "" guard: "a href=e5bebd43-6881-4c89-be4a-7cc97632df51/.a href=40897179-1953-42c5-8c8b-1b28cf0a4f21/" }
            transition "Seat TV - VOD Playing" -> "Seat TV - Displaying VOD User Interface" { id: "804c0253-839a-4e9d-8983-75897ab3fed4" trigger: "" guard: "a href=e5bebd43-6881-4c89-be4a-7cc97632df51/.a href=087ba902-96a6-4592-9945-69760178a04f/" }
        }
        mode "Seat TV - News Service Activated" {
            id: "f59d395d-fd14-479f-81ca-3546acf8dc90"
        }
        mode "Seat TV - Any Other Service Activated" {
            id: "3db2a933-36c3-4a49-a597-ae58972ffe83"
        }
        mode "Seat TV - Gaming Service Activated" {
            id: "80925aa4-9aaa-4387-9670-c62f4130fc70"
        }
        mode "Seat TV - Moving-Map Service Activated" {
            id: "159da3e6-0b3a-412d-a659-778575dbdde0"
        }
        state "Deep History" {
            id: "ad86fe5b-68f6-44d7-9c6c-a610d6291bf7"
            kind: "deep_history"
        }
        transition "Seat TV - VOD Service Activated" -> "Seat TV - Imposed Video Running" { id: "0b2784d4-883c-4838-8e51-4b4a7564ec86" trigger: "" guard: "a href=e5bebd43-6881-4c89-be4a-7cc97632df51/.a href=f28f71e8-9a80-429c-b71d-bf3ca3b46fa7/" }
    }
    mode "PTV - Interrupted" {
        id: "a8797b2f-b6f7-4bc2-9613-5fc341391c23"
        mode "Seat TV - Imposed Video Paused" {
            id: "6cfd0627-b1d6-45db-8fcf-9998fe24fdcc"
        }
        mode "Seat TV - Imposed Video Running" {
            id: "e5f3db9a-a89a-4323-a8c7-c92dcf099d71"
        }
        mode "Seat TV - Audio Announcement Running" {
            id: "0a012506-1e9a-48d0-a9a5-157c56054432"
        }
        transition "Seat TV - Imposed Video Paused" -> "Seat TV - Imposed Video Running" { id: "e6a51ff2-0533-448c-89ea-ff3ea4aea367" trigger: "" guard: "Value = PAUSE" }
    }
    transition "Seat TV - Home Page Displayed" -> "Seat TV - Moving-Map Service Activated" { id: "6a5cba3a-1af0-4222-be1f-5fc38999d490" trigger: "" }
    transition "Seat TV - Home Page Displayed" -> "Seat TV - VOD Service Activated" { id: "e5d2d3e2-1398-4e50-9252-e95ab7fc8816" trigger: "" }
    transition "Seat TV - Home Page Displayed" -> "Seat TV - News Service Activated" { id: "3834215b-f255-4c38-bcc9-1310a4c0f5fd" trigger: "" }
    transition "Seat TV - Home Page Displayed" -> "Seat TV - Gaming Service Activated" { id: "e987d235-53f5-44c5-96ec-084c9d807ff1" trigger: "" }
    transition "Seat TV - Home Page Displayed" -> "Seat TV - Any Other Service Activated" { id: "775785c2-9a6b-440b-8074-b8e66dd6be07" trigger: "" }
    transition "Seat TV - Home Page Displayed" -> "Final" { id: "c173ec6c-22c9-4539-9ed5-1b16d0479abb" trigger: "" }
    transition "PTV - Interrupted" -> "Final" { id: "e334b6e2-ced6-40a3-8520-1531b1b0e5c5" trigger: "" }
    transition "Seat TV - End-User Service Running" -> "Seat TV - Home Page Displayed" { id: "9c18de24-4abb-4915-9bf7-122c6b047f6e" trigger: "" }
    transition "Seat TV - End-User Service Running" -> "Final" { id: "a11bc62b-d94b-47ea-88c2-96bedc968cff" trigger: "" }
    transition "Seat TV - Home Page Displayed" -> "PTV - Maintenance" { id: "1d221aeb-40b6-4327-9a44-0392e58bebaf" trigger: "" }
    transition "PTV - Interrupted" -> "Deep History" { id: "b01b3b80-6a1e-46ed-883f-3169c4abf582" trigger: "" events: ["Imposed Movie Ended"] action: ["910a61f1-3da0-404f-9db5-c226af9a3c47"] }
    transition "Seat TV - End-User Service Running" -> "Seat TV - Home Page Displayed" { id: "2664016f-9157-40fe-bfeb-52783fb2b772" trigger: "" }
}

scenario "[OES] Flight Phases" {
    id: "d04ae481-72c3-432e-b4e7-b2d66264d3b0"
    owner: "2569c843-e368-451c-b7be-67af191a35a3"
    kind: "INTERACTION"
    participants: ["87bfb1b6-b85e-4497-950d-663dd78c4502", "dab8526e-cf3a-4c8a-8fa1-7df4aa281bcf", "2060479f-e59f-4b8f-975f-f05f5986e9b3"]
    state "dab8526e-cf3a-4c8a-8fa1-7df4aa281bcf" "Boarding" {
        id: "9706b9e3-221e-4d01-b4b1-44a5210c385e"
    }
    message "87bfb1b6-b85e-4497-950d-663dd78c4502" -> "dab8526e-cf3a-4c8a-8fa1-7df4aa281bcf" "Ambiance Music" { id: "eacbc4ba-4a19-4bf7-a560-f2b94a2fd187" exchange: "dec8a64b-2308-4e54-8baa-49e188b7b727" type: "async" kind: "ASYNCHRONOUS_CALL" }
    message "dab8526e-cf3a-4c8a-8fa1-7df4aa281bcf" -> "2060479f-e59f-4b8f-975f-f05f5986e9b3" "Ambiance Music" { id: "9044aa10-b576-40d3-99fa-c94d7213a208" exchange: "3b814835-c766-48ff-819a-379921f146da" type: "async" kind: "ASYNCHRONOUS_CALL" }
    state "dab8526e-cf3a-4c8a-8fa1-7df4aa281bcf" "Takeoff" {
        id: "68af75a1-747f-411b-a83f-6376871c1e1f"
    }
    message "87bfb1b6-b85e-4497-950d-663dd78c4502" -> "dab8526e-cf3a-4c8a-8fa1-7df4aa281bcf" "Safety Instructions" { id: "dca627ff-607a-485a-8ce3-4252c57720fb" exchange: "b55b0fd7-f59c-4976-aab1-2f19493cafde" type: "async" kind: "ASYNCHRONOUS_CALL" }
    message "dab8526e-cf3a-4c8a-8fa1-7df4aa281bcf" -> "2060479f-e59f-4b8f-975f-f05f5986e9b3" "Imposed Movie" { id: "358e3221-0a55-4a62-a427-a811c9b18d6b" exchange: "c80d7d2b-61d4-4c72-aaca-7055a6949c67" type: "async" kind: "ASYNCHRONOUS_CALL" }
    state "dab8526e-cf3a-4c8a-8fa1-7df4aa281bcf" "Cruising" {
        id: "cec47fb6-07fd-4058-86c6-02002abe340f"
    }
    fragment loop {
        fragment alt {
            operand {
                message "2060479f-e59f-4b8f-975f-f05f5986e9b3" -> "dab8526e-cf3a-4c8a-8fa1-7df4aa281bcf" "Movie Selection" { id: "d5ded93c-a7e8-43ba-845f-49fff43265d0" exchange: "3d34fde4-8b91-4602-be32-733f3adcc472" type: "async" kind: "ASYNCHRONOUS_CALL" }
                message "dab8526e-cf3a-4c8a-8fa1-7df4aa281bcf" -> "2060479f-e59f-4b8f-975f-f05f5986e9b3" "Chosen Movie" { id: "f786d9d9-270b-4467-858b-66ca3f129f81" exchange: "b5cf7709-6acd-4494-b28a-08f794b843ce" type: "async" kind: "ASYNCHRONOUS_CALL" }
            }
            operand {
                message "2060479f-e59f-4b8f-975f-f05f5986e9b3" -> "dab8526e-cf3a-4c8a-8fa1-7df4aa281bcf" "Music Selection" { id: "f00d1127-e1b9-4964-a1e8-dcc1d832e2f4" exchange: "b7d8100d-404a-4b7b-b08b-01f232cd0335" type: "async" kind: "ASYNCHRONOUS_CALL" }
                message "dab8526e-cf3a-4c8a-8fa1-7df4aa281bcf" -> "2060479f-e59f-4b8f-975f-f05f5986e9b3" "Chosen Music" { id: "0704cd0d-d1a2-4888-8ce6-75a650a48ac8" exchange: "195cfecc-4cf8-43e4-85ab-6cc73311e388" type: "async" kind: "ASYNCHRONOUS_CALL" }
            }
            operand {
                message "dab8526e-cf3a-4c8a-8fa1-7df4aa281bcf" -> "2060479f-e59f-4b8f-975f-f05f5986e9b3" "Moving Map" { id: "a8b539ff-4fd6-448e-82e0-b4683cc25168" exchange: "f21c4d30-f56c-412b-b7a7-1e072429076c" type: "async" kind: "ASYNCHRONOUS_CALL" }
            }
        }
    }
    message "87bfb1b6-b85e-4497-950d-663dd78c4502" -> "dab8526e-cf3a-4c8a-8fa1-7df4aa281bcf" "Audio Announcement" { id: "1cefab87-30c3-4c0d-ba3b-58887556b557" exchange: "daaa759b-bfa4-49bb-89a6-08d99b603af9" type: "async" kind: "ASYNCHRONOUS_CALL" }
    message "dab8526e-cf3a-4c8a-8fa1-7df4aa281bcf" -> "2060479f-e59f-4b8f-975f-f05f5986e9b3" "Audio Announcement" { id: "983804b7-f6a5-406f-84fb-1559509e423a" exchange: "d3db593e-dfeb-4ecf-b7cd-92b43ae785e0" type: "async" kind: "ASYNCHRONOUS_CALL" }
    state "dab8526e-cf3a-4c8a-8fa1-7df4aa281bcf" "Landing" {
        id: "4b505809-60b9-4731-9e8b-8c1daca25f80"
    }
    message "87bfb1b6-b85e-4497-950d-663dd78c4502" -> "dab8526e-cf3a-4c8a-8fa1-7df4aa281bcf" "Safety Instructions" { id: "4cf6eb58-0298-43ea-a821-09c7724e706d" exchange: "b55b0fd7-f59c-4976-aab1-2f19493cafde" type: "async" kind: "ASYNCHRONOUS_CALL" }
    message "dab8526e-cf3a-4c8a-8fa1-7df4aa281bcf" -> "2060479f-e59f-4b8f-975f-f05f5986e9b3" "Imposed Movie" { id: "ed223dae-2c7c-4862-bbcc-dcd5ff608760" exchange: "c80d7d2b-61d4-4c72-aaca-7055a6949c67" type: "async" kind: "ASYNCHRONOUS_CALL" }
    state "dab8526e-cf3a-4c8a-8fa1-7df4aa281bcf" "Disembarkation" {
        id: "4edb0e17-8877-4521-be61-f24f300b8fa5"
    }
    message "87bfb1b6-b85e-4497-950d-663dd78c4502" -> "dab8526e-cf3a-4c8a-8fa1-7df4aa281bcf" "Ambiance Music" { id: "fabdb668-504a-4f5d-8f69-acab06bfcebb" exchange: "dec8a64b-2308-4e54-8baa-49e188b7b727" type: "async" kind: "ASYNCHRONOUS_CALL" }
    message "dab8526e-cf3a-4c8a-8fa1-7df4aa281bcf" -> "2060479f-e59f-4b8f-975f-f05f5986e9b3" "Ambiance Music" { id: "24fa5910-3a21-4996-bc1c-77c49b32ffe5" exchange: "3b814835-c766-48ff-819a-379921f146da" type: "async" kind: "ASYNCHRONOUS_CALL" }
    state "dab8526e-cf3a-4c8a-8fa1-7df4aa281bcf" "Idle" {
        id: "c530462c-04f5-4861-bb67-e439fccc43ad"
    }
}

scenario "Provide Audio and Video Intercommunication Means / [FS] Perform Audio Announcement" {
    id: "fa8dc626-68de-4e83-b9b9-2a1aa7813b7c"
    owner: "291fb362-9ae5-4779-8405-1281ea3709f6"
    kind: "FUNCTIONAL"
    participants: ["e04025f9-6b52-4377-bac1-78d0bf2f8fed", "e1808524-dfa9-4883-a809-c44a87712d1b", "d8b8c1bd-a2ea-47fd-a7a9-0239408db1a5", "8f279b07-26aa-46f6-8c9a-2a132f0b1177"]
    message "8f279b07-26aa-46f6-8c9a-2a132f0b1177" -> "e1808524-dfa9-4883-a809-c44a87712d1b" "Audio Stream" { id: "edc72075-76ee-4438-afb0-8bca2536b695" exchange: "c9afe3ac-0a52-450a-b97d-741e4b03ad39" type: "async" kind: "ASYNCHRONOUS_CALL" }
    message "e1808524-dfa9-4883-a809-c44a87712d1b" -> "d8b8c1bd-a2ea-47fd-a7a9-0239408db1a5" "VOD Movie" { id: "876c4dc5-6cb2-4c6e-af94-fccfdb5a40a9" exchange: "2d2e8936-7dc3-40da-bb09-6d3f2ce71921" type: "async" kind: "ASYNCHRONOUS_CALL" }
    message "e1808524-dfa9-4883-a809-c44a87712d1b" -> "e04025f9-6b52-4377-bac1-78d0bf2f8fed" "Audio Signal" { id: "a32ac277-7c4d-4bb8-a056-6c09792746d9" exchange: "883d204e-69ff-40e9-80e4-e4d5845bb2d2" type: "async" kind: "ASYNCHRONOUS_CALL" }
}

scenario "Provide Audio and Video Intercommunication Means / [ES] Perform Audio Announcement" {
    id: "a395ed36-9833-4a4a-89cf-a4c41cd5928c"
    owner: "291fb362-9ae5-4779-8405-1281ea3709f6"
    kind: "DATA_FLOW"
    participants: ["181a678c-dca9-46c1-9d18-b5a0c457c0de", "c8b78c78-5b11-4fc0-87b7-3ca84622efea", "e0a46739-7e50-4ea6-b198-0e5efb9ce68f"]
    function "c8b78c78-5b11-4fc0-87b7-3ca84622efea" "Send Audio Announcement" {
        id: "f6ae674e-bc76-4415-bf9d-3b370521b7b4"
    }
    message "c8b78c78-5b11-4fc0-87b7-3ca84622efea" -> "e0a46739-7e50-4ea6-b198-0e5efb9ce68f" "Audio Stream" { id: "d0c5d833-8486-4daa-af83-19c08e1974ab" exchange: "c9afe3ac-0a52-450a-b97d-741e4b03ad39" type: "async" kind: "ASYNCHRONOUS_CALL" }
    function "e0a46739-7e50-4ea6-b198-0e5efb9ce68f" "Broadcast Audio Video Streams" {
        id: "c41ab0c6-142f-4c12-9aef-67c6ec936f65"
    }
    message "e0a46739-7e50-4ea6-b198-0e5efb9ce68f" -> "e0a46739-7e50-4ea6-b198-0e5efb9ce68f" "VOD Movie" { id: "07a844f4-50d5-4f82-8da8-939879447a22" exchange: "2d2e8936-7dc3-40da-bb09-6d3f2ce71921" type: "async" kind: "ASYNCHRONOUS_CALL" }
    function "e0a46739-7e50-4ea6-b198-0e5efb9ce68f" "Manage Passenger Service Interruptions" {
        id: "02ce2cd3-337e-4a20-9ed6-6f75f0eaf6ba"
    }
    message "e0a46739-7e50-4ea6-b198-0e5efb9ce68f" -> "181a678c-dca9-46c1-9d18-b5a0c457c0de" "Audio Signal" { id: "1538c46a-7924-4ff7-a37d-d1da083bc690" exchange: "883d204e-69ff-40e9-80e4-e4d5845bb2d2" type: "async" kind: "ASYNCHRONOUS_CALL" }
    function "181a678c-dca9-46c1-9d18-b5a0c457c0de" "e04025f9-6b52-4377-bac1-78d0bf2f8fed" {
        id: "bd11c34f-7979-4712-8613-9bb20537125c"
    }
}

scenario "[ES] Start VOD Service" {
    id: "58d8d80a-34ab-4e60-aee2-fb8eb885583a"
    owner: "839ba873-8442-4a01-bd1a-1f4c3cc6ed52"
    kind: "DATA_FLOW"
    participants: ["e0a46739-7e50-4ea6-b198-0e5efb9ce68f", "181a678c-dca9-46c1-9d18-b5a0c457c0de"]
    state "e0a46739-7e50-4ea6-b198-0e5efb9ce68f" "0b71ac24-f73d-4aac-a730-3d6c57ca62bc" {
        id: "b23052f7-c2f0-4109-8b04-bdcb77043017"
        function "181a678c-dca9-46c1-9d18-b5a0c457c0de" "65787ee3-ab55-4200-91e3-d772a4ee3f28" {
            id: "d74eec32-f046-4ebc-b098-3489e41e51dc"
        }
    }
    message "181a678c-dca9-46c1-9d18-b5a0c457c0de" -> "e0a46739-7e50-4ea6-b198-0e5efb9ce68f" "Passenger VOD Service Selection" { id: "a49206ca-7cae-4f83-a158-b2fcf7a8fde7" exchange: "af919966-017f-44fa-856a-f680d604ded8" type: "async" kind: "ASYNCHRONOUS_CALL" }
    function "e0a46739-7e50-4ea6-b198-0e5efb9ce68f" "Determine Service Availability" {
        id: "6a9f1b91-6ec6-44fc-9d11-cdb2fc08e114"
    }
    message "e0a46739-7e50-4ea6-b198-0e5efb9ce68f" -> "e0a46739-7e50-4ea6-b198-0e5efb9ce68f" "Service Availability" { id: "6b2041ae-5575-466d-9278-d41c86d11ac7" exchange: "6dda546d-7b7f-4ff4-939c-d9c8e441a3a8" type: "async" kind: "ASYNCHRONOUS_CALL" }
    function "e0a46739-7e50-4ea6-b198-0e5efb9ce68f" "Handle Service Activation Requests" {
        id: "4b3f2d55-3230-4819-9aab-bb20937b4c06"
    }
    message "e0a46739-7e50-4ea6-b198-0e5efb9ce68f" -> "e0a46739-7e50-4ea6-b198-0e5efb9ce68f" "Passenger VOD Service Activation" { id: "5b3e1f51-87cc-4b10-9e31-329e02c91204" exchange: "e7ca817a-2207-4beb-b7ee-98a679f3e0fb" type: "async" kind: "ASYNCHRONOUS_CALL" }
    state "e0a46739-7e50-4ea6-b198-0e5efb9ce68f" "ff567a76-0861-45b6-8d64-293758b0dd7f" {
        id: "808c3678-6822-4066-8de9-9d0df0337b3c"
    }
    message "e0a46739-7e50-4ea6-b198-0e5efb9ce68f" -> "e0a46739-7e50-4ea6-b198-0e5efb9ce68f" "Available VOD Movies List" { id: "f233dd0b-4c1d-4db0-9566-1feb1c89c4a3" exchange: "5e30c0be-9342-477c-82a1-c1d1ba0682fa" type: "async" kind: "ASYNCHRONOUS_CALL" }
    function "e0a46739-7e50-4ea6-b198-0e5efb9ce68f" "0a2099f5-2831-4beb-89ae-996a6f54d032" {
        id: "c297a010-c21e-4bcf-8f80-34b2eecb5eee"
    }
    message "e0a46739-7e50-4ea6-b198-0e5efb9ce68f" -> "181a678c-dca9-46c1-9d18-b5a0c457c0de" "Displayed VOD Movies List" { id: "544391a4-e152-415f-9650-50eb0ecb6c57" exchange: "8dcadf53-92b7-453c-a4a6-3e3780eabb82" type: "async" kind: "ASYNCHRONOUS_CALL" }
}

scenario "Provide Video Entertainment Services / [ES] Start Playing VOD Movie" {
    id: "2d18447b-6c5a-45f7-8149-245290242bab"
    owner: "839ba873-8442-4a01-bd1a-1f4c3cc6ed52"
    kind: "DATA_FLOW"
    participants: ["181a678c-dca9-46c1-9d18-b5a0c457c0de", "e0a46739-7e50-4ea6-b198-0e5efb9ce68f"]
    message "e0a46739-7e50-4ea6-b198-0e5efb9ce68f" -> "181a678c-dca9-46c1-9d18-b5a0c457c0de" "Displayed VOD Movies List" { id: "9a40b120-37e6-4e4a-be53-e674e9a990e7" exchange: "8dcadf53-92b7-453c-a4a6-3e3780eabb82" type: "async" kind: "ASYNCHRONOUS_CALL" }
    function "181a678c-dca9-46c1-9d18-b5a0c457c0de" "Command VOD Service" {
        id: "3d15f229-522b-4aa2-9278-4de68e800125"
    }
    message "181a678c-dca9-46c1-9d18-b5a0c457c0de" -> "e0a46739-7e50-4ea6-b198-0e5efb9ce68f" "VOD Movie Selection" { id: "82a8e076-8e63-4a77-a236-8a359c3a919d" exchange: "613d97e8-7adb-4f2b-a3de-359c73087bff" type: "async" kind: "ASYNCHRONOUS_CALL" }
    function "e0a46739-7e50-4ea6-b198-0e5efb9ce68f" "0a2099f5-2831-4beb-89ae-996a6f54d032" {
        id: "c8d1f9dc-e325-4363-8d82-0da62b3445ae"
    }
    message "e0a46739-7e50-4ea6-b198-0e5efb9ce68f" -> "181a678c-dca9-46c1-9d18-b5a0c457c0de" "Displayed Available Languages" { id: "6ad0d1a3-cd39-48b0-bc4d-5a9c8d08f946" exchange: "c8bfbb5b-865d-4924-902f-b209dbfb3433" type: "async" kind: "ASYNCHRONOUS_CALL" }
    message "181a678c-dca9-46c1-9d18-b5a0c457c0de" -> "e0a46739-7e50-4ea6-b198-0e5efb9ce68f" "VOD Movie Language Selection" { id: "a3e03598-02bf-4cbf-9fe0-caa7ed2221ca" exchange: "fa5c887c-feac-438f-ac22-2bd908b30483" type: "async" kind: "ASYNCHRONOUS_CALL" }
    function "e0a46739-7e50-4ea6-b198-0e5efb9ce68f" "0a2099f5-2831-4beb-89ae-996a6f54d032" {
        id: "3f919697-1b58-41f0-a218-86e0ae548f0e"
    }
    message "e0a46739-7e50-4ea6-b198-0e5efb9ce68f" -> "181a678c-dca9-46c1-9d18-b5a0c457c0de" "Displayed Movie Description" { id: "e87b645c-aa75-4cc9-901b-520615a70d4e" exchange: "728283a8-142b-4afe-b67c-f2f75cf5a3a3" type: "async" kind: "ASYNCHRONOUS_CALL" }
    message "181a678c-dca9-46c1-9d18-b5a0c457c0de" -> "e0a46739-7e50-4ea6-b198-0e5efb9ce68f" "VOD Movie Play Selection" { id: "7a932513-fef5-45c1-af63-48739e1f3ddf" exchange: "0ce48a3f-6a6d-498c-b41e-280d1bc64928" type: "async" kind: "ASYNCHRONOUS_CALL" }
    function "e0a46739-7e50-4ea6-b198-0e5efb9ce68f" "Handle VOD Movie Controls" {
        id: "3fe00675-7353-4674-a2a8-1ce3400d5322"
    }
    message "e0a46739-7e50-4ea6-b198-0e5efb9ce68f" -> "e0a46739-7e50-4ea6-b198-0e5efb9ce68f" "VOD Movie Control Command" { id: "99a9473c-7a5a-42e3-9faf-393d1fc97b33" exchange: "4125bbcd-f124-4bd5-9f40-aa73c538a3e4" type: "async" kind: "ASYNCHRONOUS_CALL" }
    function "e0a46739-7e50-4ea6-b198-0e5efb9ce68f" "Broadcast Audio Video Streams" {
        id: "783f0f07-87cc-4ad7-9478-1600d18ccdd8"
    }
    message "e0a46739-7e50-4ea6-b198-0e5efb9ce68f" -> "181a678c-dca9-46c1-9d18-b5a0c457c0de" "Played Audio Video" { id: "c4ed4216-0963-4bfb-92d6-af9263a1e7fa" exchange: "15c3b494-a0b8-4aca-b734-01978120cbd4" type: "async" kind: "ASYNCHRONOUS_CALL" }
    function "181a678c-dca9-46c1-9d18-b5a0c457c0de" "Watch Video on Private Screen" {
        id: "e4b9cc49-172d-4322-bf6a-638555c4005d"
    }
}

scenario "Provide Access Management Control / [FS] Determine Passenger Service Availability" {
    id: "ef6ce369-4a17-4655-aa96-66b3b69c7315"
    owner: "74e6e35e-b1e7-45c1-bd9f-5d80c0714472"
    kind: "FUNCTIONAL"
    participants: ["65787ee3-ab55-4200-91e3-d772a4ee3f28", "2c453928-a098-4f8a-a1e6-fc545a2840d1", "b7244d6a-08ab-4455-bbd9-aa5f1bbc9b5b", "3f8567f2-5389-42c0-84a9-8a1360dc9572", "0a2099f5-2831-4beb-89ae-996a6f54d032"]
    message "3f8567f2-5389-42c0-84a9-8a1360dc9572" -> "b7244d6a-08ab-4455-bbd9-aa5f1bbc9b5b" "IFE Operating Profile" { id: "7f5d726b-a2c2-424f-ba08-c3fcd8b99714" exchange: "e3c0b2f5-ec50-4dba-9808-04d6179ad01e" type: "async" kind: "ASYNCHRONOUS_CALL" }
    message "65787ee3-ab55-4200-91e3-d772a4ee3f28" -> "2c453928-a098-4f8a-a1e6-fc545a2840d1" "Passenger VOD Service Selection" { id: "b2e20ae5-78d5-4dfa-98ac-be707f3b8911" exchange: "af919966-017f-44fa-856a-f680d604ded8" type: "async" kind: "ASYNCHRONOUS_CALL" }
    message "b7244d6a-08ab-4455-bbd9-aa5f1bbc9b5b" -> "2c453928-a098-4f8a-a1e6-fc545a2840d1" "Service Availability" { id: "7fed5781-184e-496d-a6aa-b399b04726c3" exchange: "6dda546d-7b7f-4ff4-939c-d9c8e441a3a8" type: "async" kind: "ASYNCHRONOUS_CALL" }
    message "2c453928-a098-4f8a-a1e6-fc545a2840d1" -> "0a2099f5-2831-4beb-89ae-996a6f54d032" "Passenger VOD Service Activation" { id: "50623301-5554-4c93-b8c7-3601db2a87ff" exchange: "e7ca817a-2207-4beb-b7ee-98a679f3e0fb" type: "async" kind: "ASYNCHRONOUS_CALL" }
}

scenario "Integrate Aircraft Constraints / [FS] Handle Cabin Decompression Event" {
    id: "3324a066-17a5-4b28-9dac-ad81cd4e7893"
    owner: "5acd248a-3961-4e71-85a4-5bc3b775215f"
    kind: "FUNCTIONAL"
    participants: ["9972b7a8-df8a-4626-b298-1dd0d19e451e", "3f8567f2-5389-42c0-84a9-8a1360dc9572", "b7244d6a-08ab-4455-bbd9-aa5f1bbc9b5b"]
    state "b7244d6a-08ab-4455-bbd9-aa5f1bbc9b5b" "6171dcbd-85b7-4452-a035-8ab9c1be37e7" {
        id: "ca82f34d-1a84-4176-9c36-8e85c6e42276"
    }
    message "9972b7a8-df8a-4626-b298-1dd0d19e451e" -> "3f8567f2-5389-42c0-84a9-8a1360dc9572" "Decompression Notification" { id: "42a1df57-f193-401a-8dde-26d603453c60" exchange: "49328560-baa6-492c-ad89-8166ce70fdad" type: "async" kind: "ASYNCHRONOUS_CALL" }
    message "3f8567f2-5389-42c0-84a9-8a1360dc9572" -> "b7244d6a-08ab-4455-bbd9-aa5f1bbc9b5b" "IFE Operating Profile" { id: "46a61a97-ae94-4ef7-838c-d0735180cccd" exchange: "e3c0b2f5-ec50-4dba-9808-04d6179ad01e" type: "async" kind: "ASYNCHRONOUS_CALL" }
    state "b7244d6a-08ab-4455-bbd9-aa5f1bbc9b5b" "75503038-7ad6-4212-9098-b5764b7a430b" {
        id: "fe50d035-faca-4e09-be39-99ce97e94029"
    }
}

scenario "Provide Audio and Video Intercommunication Means / [FS] Perform Audio Announcement (2)" {
    id: "9c08879f-39bb-42a1-b1bf-badb30f67d12"
    owner: "e373786f-d962-4a05-9360-902a524b7522"
    kind: "FUNCTIONAL"
    participants: ["c6ed6258-be4c-4784-a6e4-3e95f9f22dc7", "ee920873-7e2d-487f-a51e-76236d6b094e", "b4623d17-6e62-411d-839c-2d768664a920", "910a61f1-3da0-404f-9db5-c226af9a3c47", "3136218c-bf14-437d-99c9-6593352eead9", "39f96e89-8ead-4742-bc1f-6eb0dd33990b", "7b604e03-85be-4d93-8035-8a84ef8086db", "4ff700ef-4f6b-4151-917b-e59064e5a225", "30d03eaa-2615-481c-bf1f-53f177d8147c", "18da001e-dd8b-4041-ba4c-eb2211c624c4"]
    message "c6ed6258-be4c-4784-a6e4-3e95f9f22dc7" -> "ee920873-7e2d-487f-a51e-76236d6b094e" "Audio Stream" { id: "372d85f5-18a7-47b4-8230-1bb2b69537b8" exchange: "0ad613f5-c36e-4cb8-b9f9-ce38b8078929" type: "async" kind: "ASYNCHRONOUS_CALL" }
    message "ee920873-7e2d-487f-a51e-76236d6b094e" -> "39f96e89-8ead-4742-bc1f-6eb0dd33990b" "Audio Stream" { id: "4da0d92d-a724-420c-9f7d-90e792bbffc4" exchange: "83a3c9ae-1cb3-4cab-885f-de529d4585ea" type: "async" kind: "ASYNCHRONOUS_CALL" }
    fragment par {
        operand {
            message "39f96e89-8ead-4742-bc1f-6eb0dd33990b" -> "7b604e03-85be-4d93-8035-8a84ef8086db" "Audio Stream" { id: "107eb049-c6b7-49e2-99fd-cc0f864e0d9c" exchange: "cfbfc6da-1898-4a5d-8a93-84865d97a1b3" type: "async" kind: "ASYNCHRONOUS_CALL" }
            message "7b604e03-85be-4d93-8035-8a84ef8086db" -> "18da001e-dd8b-4041-ba4c-eb2211c624c4" "Audio Signal" { id: "d7013729-dabc-48b0-a7dd-36ab7204de85" exchange: "2c650bc2-d254-4026-b5cf-c2a001f289bd" type: "async" kind: "ASYNCHRONOUS_CALL" }
        }
        operand {
            message "39f96e89-8ead-4742-bc1f-6eb0dd33990b" -> "b4623d17-6e62-411d-839c-2d768664a920" "Service Interruption Command" { id: "7e4f423d-f102-46f3-bc35-ff75f16dbad2" exchange: "9a9a0ba6-1725-4026-92da-dcb18e1b0f4e" type: "async" kind: "ASYNCHRONOUS_CALL" }
            message "b4623d17-6e62-411d-839c-2d768664a920" -> "4ff700ef-4f6b-4151-917b-e59064e5a225" "Audio Interruption Notification" { id: "8f208f50-4461-4462-bb40-be7034ef967a" exchange: "b00d3b7f-56ac-4491-921e-fe1785c1c228" type: "async" kind: "ASYNCHRONOUS_CALL" }
            message "4ff700ef-4f6b-4151-917b-e59064e5a225" -> "18da001e-dd8b-4041-ba4c-eb2211c624c4" "Displayed Audio Interruption Screen" { id: "541cebf2-65a8-49ee-80c8-89ad3abd7f60" exchange: "d8ae3120-7282-4314-bf35-41dd4c2e82f6" type: "async" kind: "ASYNCHRONOUS_CALL" }
            message "b4623d17-6e62-411d-839c-2d768664a920" -> "910a61f1-3da0-404f-9db5-c226af9a3c47" "Service Interruption Command" { id: "8cdf93eb-f77b-44b1-b958-f5cb2b79230b" exchange: "fc741435-84fa-457c-b116-13d8c0f15249" type: "async" kind: "ASYNCHRONOUS_CALL" }
            message "910a61f1-3da0-404f-9db5-c226af9a3c47" -> "3136218c-bf14-437d-99c9-6593352eead9" "VOD Movie Interruption Command" { id: "e48d7141-e11f-41d0-b8e5-656c2ecc373e" exchange: "4aee9cf0-9b14-436e-86ad-5a11d4866e40" type: "async" kind: "ASYNCHRONOUS_CALL" }
        }
    }
}

scenario "Provide Audio and Video Intercommunication Means / [ES] Perform Audio Announcement (2)" {
    id: "2c5abb7e-3281-4651-8458-39d383b9dcc0"
    owner: "e373786f-d962-4a05-9360-902a524b7522"
    kind: "DATA_FLOW"
    participants: ["a0ceff37-72a6-4da7-a646-89425a748235", "_SF1a8JY-Ed6rH5SGn-UjSg", "e02be1c5-a1a2-4c11-be30-259e92ef5826", "_2SUXEJZFEd6rH5SGn-UjSg", "df043998-7fd1-4d4d-899f-7257788d644f", "_nxYVII2dEd6Q36a6kYkaHQ", "281c331f-1184-4c7d-afb2-7a55ff09a80f", "395ee0ab-cb3d-48e8-a411-09a98053292f"]
    message "a0ceff37-72a6-4da7-a646-89425a748235" -> "_SF1a8JY-Ed6rH5SGn-UjSg" "Audio Stream" { id: "200db697-eb4c-47a7-844f-070de17065af" exchange: "0ad613f5-c36e-4cb8-b9f9-ce38b8078929" type: "async" kind: "ASYNCHRONOUS_CALL" }
    function "_SF1a8JY-Ed6rH5SGn-UjSg" "ee920873-7e2d-487f-a51e-76236d6b094e" {
        id: "0243e1e0-ad95-4c47-9ca1-a480c312da19"
    }
    message "_SF1a8JY-Ed6rH5SGn-UjSg" -> "df043998-7fd1-4d4d-899f-7257788d644f" "Audio Stream" { id: "a8a2d155-6cee-4de8-9cb1-0371ba6d9d87" exchange: "83a3c9ae-1cb3-4cab-885f-de529d4585ea" type: "async" kind: "ASYNCHRONOUS_CALL" }
    function "df043998-7fd1-4d4d-899f-7257788d644f" "39f96e89-8ead-4742-bc1f-6eb0dd33990b" {
        id: "043db20f-c0f1-49a0-bfdc-cb8227fd265f"
    }
    fragment par {
        operand {
            message "df043998-7fd1-4d4d-899f-7257788d644f" -> "_nxYVII2dEd6Q36a6kYkaHQ" "Audio Stream" { id: "4adcbb68-7fd4-442e-8ea8-6696b7e689a3" exchange: "cfbfc6da-1898-4a5d-8a93-84865d97a1b3" type: "async" kind: "ASYNCHRONOUS_CALL" }
            function "_nxYVII2dEd6Q36a6kYkaHQ" "Play Audio-Video Stream on Seat TV" {
                id: "e5f623d5-118b-4e6d-9227-f7fc0f698799"
            }
            message "_nxYVII2dEd6Q36a6kYkaHQ" -> "395ee0ab-cb3d-48e8-a411-09a98053292f" "Audio Signal" { id: "dcebbeba-e659-4d56-9bb4-6438cd8ef5ed" exchange: "2c650bc2-d254-4026-b5cf-c2a001f289bd" type: "async" kind: "ASYNCHRONOUS_CALL" }
            function "395ee0ab-cb3d-48e8-a411-09a98053292f" "18da001e-dd8b-4041-ba4c-eb2211c624c4" {
                id: "a2931dbb-2fa6-484e-a5ab-4a49c997b7ef"
            }
        }
        operand {
            message "df043998-7fd1-4d4d-899f-7257788d644f" -> "e02be1c5-a1a2-4c11-be30-259e92ef5826" "Service Interruption Command" { id: "5fb606e6-5ac5-4fa1-b1e8-2e862903dff5" exchange: "9a9a0ba6-1725-4026-92da-dcb18e1b0f4e" type: "async" kind: "ASYNCHRONOUS_CALL" }
            function "e02be1c5-a1a2-4c11-be30-259e92ef5826" "b4623d17-6e62-411d-839c-2d768664a920" {
                id: "b3c0e7d2-7efe-4dc1-aef5-1290f78e74d3"
            }
            message "e02be1c5-a1a2-4c11-be30-259e92ef5826" -> "281c331f-1184-4c7d-afb2-7a55ff09a80f" "Audio Interruption Notification" { id: "8f0a0ee6-dcfc-4234-9fff-d687cc824caa" exchange: "b00d3b7f-56ac-4491-921e-fe1785c1c228" type: "async" kind: "ASYNCHRONOUS_CALL" }
            function "281c331f-1184-4c7d-afb2-7a55ff09a80f" "4ff700ef-4f6b-4151-917b-e59064e5a225" {
                id: "b632263d-1168-4b1b-abba-e437ec3e9539"
            }
            message "281c331f-1184-4c7d-afb2-7a55ff09a80f" -> "395ee0ab-cb3d-48e8-a411-09a98053292f" "Displayed Audio Interruption Screen" { id: "00312aea-84f4-4edb-992b-3d0872cd260f" exchange: "d8ae3120-7282-4314-bf35-41dd4c2e82f6" type: "async" kind: "ASYNCHRONOUS_CALL" }
            function "395ee0ab-cb3d-48e8-a411-09a98053292f" "18da001e-dd8b-4041-ba4c-eb2211c624c4" {
                id: "bb693265-666a-4aa6-933e-75d28d01c34f"
            }
            message "e02be1c5-a1a2-4c11-be30-259e92ef5826" -> "_2SUXEJZFEd6rH5SGn-UjSg" "Service Interruption Command" { id: "efde56b9-a41b-4032-8350-f4c6d433c4e9" exchange: "fc741435-84fa-457c-b116-13d8c0f15249" type: "async" kind: "ASYNCHRONOUS_CALL" }
            function "_2SUXEJZFEd6rH5SGn-UjSg" "910a61f1-3da0-404f-9db5-c226af9a3c47" {
                id: "d0ea4a55-9463-4529-8c57-974e4c8c9f3f"
            }
            message "_2SUXEJZFEd6rH5SGn-UjSg" -> "_nxYVII2dEd6Q36a6kYkaHQ" "VOD Movie Interruption Command" { id: "a3d46f24-dffc-466d-b666-6c8f17fc7706" exchange: "4aee9cf0-9b14-436e-86ad-5a11d4866e40" type: "async" kind: "ASYNCHRONOUS_CALL" }
            function "_nxYVII2dEd6Q36a6kYkaHQ" "Store Interrupted VOD Movie Status" {
                id: "371adfce-5e7b-4667-bf55-a63b93754768"
            }
        }
    }
}

scenario "Provide Video Entertainment Services / [ES] Start Playing VOD Movie (2)" {
    id: "24e49b35-c557-4e4f-a2a7-516009925532"
    owner: "58c038e4-77d4-4b0e-9a5d-7f4253285be2"
    kind: "DATA_FLOW"
    participants: ["395ee0ab-cb3d-48e8-a411-09a98053292f", "281c331f-1184-4c7d-afb2-7a55ff09a80f", "_2SUXEJZFEd6rH5SGn-UjSg", "_nxYVII2dEd6Q36a6kYkaHQ", "_QHUAIJY7Ed6rH5SGn-UjSg"]
    message "395ee0ab-cb3d-48e8-a411-09a98053292f" -> "281c331f-1184-4c7d-afb2-7a55ff09a80f" "VOD Movie Play Selection" { id: "8221239e-b2c6-45e9-89b1-81fc481047cc" exchange: "91642f3d-3687-43e4-946c-71541aa590c5" type: "async" kind: "ASYNCHRONOUS_CALL" }
    function "281c331f-1184-4c7d-afb2-7a55ff09a80f" "5165a225-e662-4b4a-95cd-c94fc5726524" {
        id: "a5c4a2e8-6346-4d0d-815f-7092ab984091"
    }
    message "281c331f-1184-4c7d-afb2-7a55ff09a80f" -> "_2SUXEJZFEd6rH5SGn-UjSg" "VOD Movie Play Selection" { id: "5c4968ce-49f9-4dd6-9984-f9996ad7c16c" exchange: "becb1a69-c67a-40b4-8d9e-2ff2c0f6761e" type: "async" kind: "ASYNCHRONOUS_CALL" }
    function "_2SUXEJZFEd6rH5SGn-UjSg" "faf3223f-3f47-4ad6-9c58-353b34e1a30f" {
        id: "90b05613-bd49-4d43-869e-1361ad209084"
    }
    message "_2SUXEJZFEd6rH5SGn-UjSg" -> "_QHUAIJY7Ed6rH5SGn-UjSg" "VOD Movie Control Command" { id: "b2fb3704-2d65-4d60-b04b-77a106399e61" exchange: "df26fc8b-2837-40a0-ac8a-96ad5c39f9ac" type: "async" kind: "ASYNCHRONOUS_CALL" }
    function "_QHUAIJY7Ed6rH5SGn-UjSg" "Broadcast Stored Audio and Video" {
        id: "0c95ef6d-794a-4387-9b84-8f36d0d5f82f"
    }
    message "_QHUAIJY7Ed6rH5SGn-UjSg" -> "_nxYVII2dEd6Q36a6kYkaHQ" "Audio-Video Stream" { id: "ec776ff1-b035-4521-a5e0-6493d30c4228" exchange: "845007fe-fe45-46ec-a109-47742ac52529" type: "async" kind: "ASYNCHRONOUS_CALL" }
    function "_nxYVII2dEd6Q36a6kYkaHQ" "Play Audio-Video Stream on Seat TV" {
        id: "4d943a5f-2a30-4c6e-874f-b0e76e16a5b7"
    }
    message "_nxYVII2dEd6Q36a6kYkaHQ" -> "395ee0ab-cb3d-48e8-a411-09a98053292f" "Played Audio Video" { id: "6c24792e-05bc-4337-8088-4e08a780b6b9" exchange: "654aacca-49b2-49fc-949c-ed6a871c6e42" type: "async" kind: "ASYNCHRONOUS_CALL" }
    function "395ee0ab-cb3d-48e8-a411-09a98053292f" "aeafffae-c0e8-45a3-9ad1-105825574ced" {
        id: "b08927c4-079a-4bcc-a9e7-d45579bb5e18"
    }
}

scenario "[ES] Select VOD Movie" {
    id: "eb17417d-8f56-40bd-9adc-b76aeb8c0d54"
    owner: "58c038e4-77d4-4b0e-9a5d-7f4253285be2"
    kind: "DATA_FLOW"
    participants: ["395ee0ab-cb3d-48e8-a411-09a98053292f", "281c331f-1184-4c7d-afb2-7a55ff09a80f", "_2SUXEJZFEd6rH5SGn-UjSg"]
    message "281c331f-1184-4c7d-afb2-7a55ff09a80f" -> "395ee0ab-cb3d-48e8-a411-09a98053292f" "Displayed Movies List" { id: "ffcd8bdf-447e-438b-b2cd-06df9871e6d8" exchange: "6f60db60-8fc5-427f-9b15-fc4a6373d642" type: "async" kind: "ASYNCHRONOUS_CALL" }
    function "395ee0ab-cb3d-48e8-a411-09a98053292f" "bcb08500-a828-43b0-ac8b-ff4c7cb3b6b4" {
        id: "b1409ece-e009-4984-933b-1f106823c0f4"
    }
    message "395ee0ab-cb3d-48e8-a411-09a98053292f" -> "281c331f-1184-4c7d-afb2-7a55ff09a80f" "VOD Movie Selection" { id: "5d464e41-468a-486f-ad07-db32dfe12a27" exchange: "6d34ef5b-feba-4360-8812-382fe34c0bd3" type: "async" kind: "ASYNCHRONOUS_CALL" }
    function "281c331f-1184-4c7d-afb2-7a55ff09a80f" "5165a225-e662-4b4a-95cd-c94fc5726524" {
        id: "3faccca1-cc22-4f38-aa98-cbeb1ed954fe"
    }
    message "281c331f-1184-4c7d-afb2-7a55ff09a80f" -> "_2SUXEJZFEd6rH5SGn-UjSg" "VOD Movie Data Request" { id: "524e7e9f-13d9-44f7-9eb6-ba6d4810be4c" exchange: "816d9b3e-7345-4b2b-b67f-1628d8034357" type: "async" kind: "ASYNCHRONOUS_CALL" }
    function "_2SUXEJZFEd6rH5SGn-UjSg" "651179c3-78b6-4a6f-a174-74405b81b36a" {
        id: "297ada98-86c0-45d4-96cf-53051eb4badb"
    }
    message "_2SUXEJZFEd6rH5SGn-UjSg" -> "281c331f-1184-4c7d-afb2-7a55ff09a80f" "Available Languages List" { id: "2a8faf6a-761f-4391-a456-b662032d5763" exchange: "8e23a789-19bb-4c39-85ab-ada245047400" type: "async" kind: "ASYNCHRONOUS_CALL" }
    function "281c331f-1184-4c7d-afb2-7a55ff09a80f" "94aebe8b-af13-4a7a-a7ee-8fd10a7fe489" {
        id: "b65b7904-7f3b-4461-a662-9265c81d74f5"
    }
    message "281c331f-1184-4c7d-afb2-7a55ff09a80f" -> "395ee0ab-cb3d-48e8-a411-09a98053292f" "Displayed Available Languages" { id: "8787dd1f-9cca-409c-817a-d58715fd0839" exchange: "6265c622-964e-4caa-bf7d-49a2d4b48ad0" type: "async" kind: "ASYNCHRONOUS_CALL" }
    message "395ee0ab-cb3d-48e8-a411-09a98053292f" -> "281c331f-1184-4c7d-afb2-7a55ff09a80f" "VOD Movie Language Selection" { id: "ccfafa9e-9db4-4314-a65a-68f17cdaf528" exchange: "cdc4500f-0b03-4016-97eb-6094a12202aa" type: "async" kind: "ASYNCHRONOUS_CALL" }
    function "281c331f-1184-4c7d-afb2-7a55ff09a80f" "5165a225-e662-4b4a-95cd-c94fc5726524" {
        id: "56b8bce0-d20b-48f5-b99f-505918c09f41"
    }
    message "281c331f-1184-4c7d-afb2-7a55ff09a80f" -> "_2SUXEJZFEd6rH5SGn-UjSg" "VOD Movie Data Request" { id: "0806c4df-475a-44bf-ab54-cacd7985c456" exchange: "816d9b3e-7345-4b2b-b67f-1628d8034357" type: "async" kind: "ASYNCHRONOUS_CALL" }
    function "_2SUXEJZFEd6rH5SGn-UjSg" "651179c3-78b6-4a6f-a174-74405b81b36a" {
        id: "007fc57b-9d72-4709-aa7f-998a30313232"
    }
    message "_2SUXEJZFEd6rH5SGn-UjSg" -> "281c331f-1184-4c7d-afb2-7a55ff09a80f" "Movie Description" { id: "34a0567f-51a8-4107-ac4e-893275fbd648" exchange: "bbabfba6-7fbf-47e0-a5db-d14b1c72f798" type: "async" kind: "ASYNCHRONOUS_CALL" }
    function "281c331f-1184-4c7d-afb2-7a55ff09a80f" "94aebe8b-af13-4a7a-a7ee-8fd10a7fe489" {
        id: "b02aa852-42e2-45f1-8255-93504c5c602f"
    }
    message "281c331f-1184-4c7d-afb2-7a55ff09a80f" -> "395ee0ab-cb3d-48e8-a411-09a98053292f" "Displayed Movie Description" { id: "b7fb73fd-3fd5-4cfe-9fd4-c791f5e77506" exchange: "07d33b69-6881-40bd-9573-0f2d3e790544" type: "async" kind: "ASYNCHRONOUS_CALL" }
}

scenario "Integrate Aircraft Constraints / [FS] Handle Cabin Decompression Event (2)" {
    id: "7aa9d9ea-a4f4-407a-9431-c4a5bdd75af5"
    owner: "187c2207-0995-4241-b8a2-d45550185dff"
    kind: "FUNCTIONAL"
    participants: ["4b01ac5e-7920-491d-af35-1846163e5ad9", "1e812d3d-6c35-42c9-80fa-eb64dfc1a3a8", "5a58fd72-1b57-41dc-8b7b-b0b8cf577236"]
    state "5a58fd72-1b57-41dc-8b7b-b0b8cf577236" "39eb29ec-2085-47fe-b3ea-55027fc82faf" {
        id: "f91174fd-7900-492c-ae4d-f62f7a4f31e3"
    }
    message "4b01ac5e-7920-491d-af35-1846163e5ad9" -> "1e812d3d-6c35-42c9-80fa-eb64dfc1a3a8" "Decompression Notification" { id: "2539ec2f-50ae-4833-b6a0-993f758363da" exchange: "c385bac1-ff71-4f7c-b203-40e7779a63f8" type: "async" kind: "ASYNCHRONOUS_CALL" }
    message "1e812d3d-6c35-42c9-80fa-eb64dfc1a3a8" -> "5a58fd72-1b57-41dc-8b7b-b0b8cf577236" "IFE Operating Profile" { id: "66210a18-ddd5-453f-a416-322cd91d3bf6" exchange: "8c966c78-51fb-4436-b36e-cc857783924d" type: "async" kind: "ASYNCHRONOUS_CALL" }
    state "5a58fd72-1b57-41dc-8b7b-b0b8cf577236" "6d479123-ab40-4f72-a21d-b28be1c0ad54" {
        id: "bd8b1e85-d38a-494c-bbc6-4bf415d02de1"
    }
}

scenario "Provide Access Management Control / [FS] Determine Passenger Service Availability (2)" {
    id: "11c37a08-241b-462b-a3ca-0d0e29cb0f7a"
    owner: "4813aae0-442e-4b79-9d9f-cc2f236dbba4"
    kind: "FUNCTIONAL"
    participants: ["63136eb0-622b-43cc-b4da-55b86ae281c1", "2ea2703a-6dd1-4835-aaae-16179864ac1d", "c3b17f24-ae52-413f-ace7-6baab1287b02", "3e9ccc30-596e-40e6-8941-64838248dcee", "5a58fd72-1b57-41dc-8b7b-b0b8cf577236", "1e812d3d-6c35-42c9-80fa-eb64dfc1a3a8", "910a61f1-3da0-404f-9db5-c226af9a3c47", "651179c3-78b6-4a6f-a174-74405b81b36a"]
    message "1e812d3d-6c35-42c9-80fa-eb64dfc1a3a8" -> "5a58fd72-1b57-41dc-8b7b-b0b8cf577236" "IFE Operating Profile" { id: "42a20c8a-98fe-4326-82ef-1bab3e5b4b9e" exchange: "8c966c78-51fb-4436-b36e-cc857783924d" type: "async" kind: "ASYNCHRONOUS_CALL" }
    message "63136eb0-622b-43cc-b4da-55b86ae281c1" -> "2ea2703a-6dd1-4835-aaae-16179864ac1d" "VOD Service Selection" { id: "c5c0ec4f-1608-4e69-96c5-8f7bebae7326" exchange: "77bbd9a8-d804-4b57-aab6-77bd6c5add42" type: "async" kind: "ASYNCHRONOUS_CALL" }
    message "2ea2703a-6dd1-4835-aaae-16179864ac1d" -> "c3b17f24-ae52-413f-ace7-6baab1287b02" "VOD Service Activation Command" { id: "79dbfed9-15fc-4fc5-bba7-bef75c15e7ca" exchange: "cbc21308-951f-4d40-b2e8-50d8816a1d5d" type: "async" kind: "ASYNCHRONOUS_CALL" }
    message "5a58fd72-1b57-41dc-8b7b-b0b8cf577236" -> "c3b17f24-ae52-413f-ace7-6baab1287b02" "Available Passenger Services List" { id: "a16f7e87-bb80-4048-93e4-7eca369469cc" exchange: "eb002c8e-031e-4129-8a22-a4f175353b9e" type: "async" kind: "ASYNCHRONOUS_CALL" }
    message "c3b17f24-ae52-413f-ace7-6baab1287b02" -> "910a61f1-3da0-404f-9db5-c226af9a3c47" "Passenger VOD Service Activation" { id: "8baca681-d44b-4546-81fe-be1eca6e3c86" exchange: "89c75525-66d8-4da4-a704-9641017c30d1" type: "async" kind: "ASYNCHRONOUS_CALL" }
    message "910a61f1-3da0-404f-9db5-c226af9a3c47" -> "651179c3-78b6-4a6f-a174-74405b81b36a" "VOD Movie List Request" { id: "55b44af3-0bf8-4c60-a4a4-a165bbe2a533" exchange: "793b6f83-aadc-4b2e-804d-26190aab3e1d" type: "async" kind: "ASYNCHRONOUS_CALL" }
}

trace "e5d905f2-f0a3-48b6-9658-cecb923e6fb3" realizes "2a07f10a-8c7b-4065-8d35-bf2b540c8348"
trace "d301f697-5975-41e2-97af-2aea81db5f6f" realizes "b417cf33-5d66-4249-9949-f547f7876a19"
trace "e02cf446-19e2-4b24-b72d-340fd3c39355" realizes "6a78f28a-a27b-4f42-a002-04924ee4c75f"
trace "2e048fef-989b-4332-ae3c-e47c63bbec2b" realizes "a7059561-22f7-4589-aed8-05b33d8600c8"
trace "c9e7c37a-e940-434c-9dab-1f1029f2cf14" realizes "fbed3075-3536-4603-9089-f9571423c7cd"
trace "93f522c4-99eb-40b2-9baa-a576b6a3af30" realizes "a990b5ef-faa7-453d-b1f0-d46e8a15c4fd"
trace "1d278944-5217-4004-a90d-c38a21cfd0a9" realizes "74a5ce53-1394-4821-9cbf-c880d83a07dd"
trace "eb58eec3-b4ee-4a51-80c2-666fae882c3f" realizes "dd056c2f-d8b0-4f9e-b4ba-382926d2aed4"
trace "8b064948-3736-4410-ab07-4d09ca2c1d24" realizes "0ba002c1-c0d9-4e1e-98cf-60e55d02ecfd"
trace "940ce3fc-ef6f-4db0-85c2-d9866a9472a7" realizes "6c858859-acc3-4f31-a6fc-0aee4f8c07d8"
trace "fbfc173b-0081-4501-9d92-f026ff210ef2" realizes "44f6d074-c280-4d8d-83e0-ced6ba8d5cf9"
trace "7fef079a-4ee1-44ea-9388-ecaefb276599" realizes "44f6d074-c280-4d8d-83e0-ced6ba8d5cf9"
trace "cfde8bb1-4df9-4573-a516-41df984ac753" realizes "7915575a-916b-49ed-a5f0-c782e9135d68"
trace "880520c2-6bd9-4481-b9da-6ccee7b9d665" realizes "f9cabb24-100b-4e68-930a-8f35ec38c106"
trace "1bc70a69-c6ad-4203-abb9-88b644ac0b95" realizes "f9cabb24-100b-4e68-930a-8f35ec38c106"
trace "4dbd5747-9662-4378-88cf-270b69ef3827" realizes "f9cabb24-100b-4e68-930a-8f35ec38c106"
trace "e04025f9-6b52-4377-bac1-78d0bf2f8fed" realizes "60dc5173-5420-44c0-bedf-ad3f6958a79d"
trace "6ce8f26a-ef90-4420-94e1-acf6b2393720" realizes "426f80c2-78ad-4e94-8e5b-da19f22aca26"
trace "8f279b07-26aa-46f6-8c9a-2a132f0b1177" realizes "426f80c2-78ad-4e94-8e5b-da19f22aca26"
trace "6b9e59a9-89f2-497e-a348-3aef80689e61" realizes "cc39bfa2-731b-448d-a8b7-a45d1cf45a97"
trace "4d0046b4-4865-438d-b09b-fd95671db09b" realizes "1cc1517f-4971-4fbd-be1c-5c83631319a5"
trace "5e0fff47-a1a5-4a13-867c-18348bd894af" realizes "79a50368-c014-4410-aebd-627426b62614"
trace "6b54b12a-50f8-4ce5-b935-08edbf655d1f" realizes "7f5d8bbc-eca5-4656-a365-757543a38ab3"
trace "26cd71b0-ec92-41a9-b9fb-6878beede810" realizes "7f5d8bbc-eca5-4656-a365-757543a38ab3"
trace "7b7b0d2c-27e8-4ca0-8a5f-2790a37e97b6" realizes "cc39bfa2-731b-448d-a8b7-a45d1cf45a97"
trace "7b7b0d2c-27e8-4ca0-8a5f-2790a37e97b6" realizes "7f5d8bbc-eca5-4656-a365-757543a38ab3"
trace "ba4d638d-e6bb-4908-94f4-eb974405c733" realizes "cc39bfa2-731b-448d-a8b7-a45d1cf45a97"
trace "ba4d638d-e6bb-4908-94f4-eb974405c733" realizes "7f5d8bbc-eca5-4656-a365-757543a38ab3"
trace "181a678c-dca9-46c1-9d18-b5a0c457c0de" realizes "2060479f-e59f-4b8f-975f-f05f5986e9b3"
trace "4890eae8-7a62-4d7b-bf8a-1590a199eebf" realizes "87bfb1b6-b85e-4497-950d-663dd78c4502"
trace "c8b78c78-5b11-4fc0-87b7-3ca84622efea" realizes "dab8526e-cf3a-4c8a-8fa1-7df4aa281bcf"
trace "d475ed7d-4644-4461-97d4-b52772c9b425" realizes "d7a95c22-1622-41d5-8201-37cd463075c7"
trace "ee920873-7e2d-487f-a51e-76236d6b094e.FIP 1" realizes "e1808524-dfa9-4883-a809-c44a87712d1b.FIP 5"
trace "4b01ac5e-7920-491d-af35-1846163e5ad9.FOP 1" realizes "9972b7a8-df8a-4626-b298-1dd0d19e451e.FOP 1"
trace "4b01ac5e-7920-491d-af35-1846163e5ad9.FOP 2" realizes "135e77a7-79ef-45e8-b138-67e283a14cdd.FOP 1"
trace "30d03eaa-2615-481c-bf1f-53f177d8147c.FIP 1" realizes "6b9e59a9-89f2-497e-a348-3aef80689e61.FIP 1"
trace "1e812d3d-6c35-42c9-80fa-eb64dfc1a3a8.FIP 1" realizes "3f8567f2-5389-42c0-84a9-8a1360dc9572.FIP 1"
trace "1e812d3d-6c35-42c9-80fa-eb64dfc1a3a8.FIP 2" realizes "3f8567f2-5389-42c0-84a9-8a1360dc9572.FIP 2"
trace "1e812d3d-6c35-42c9-80fa-eb64dfc1a3a8.FOP 1" realizes "3f8567f2-5389-42c0-84a9-8a1360dc9572.FOP 1"
trace "70d62f70-d0b6-491c-9e9e-f5ed77179ce4.FIP 1" realizes "a16d1d72-9db2-4554-af57-411b29793904.FIP 1"
trace "70d62f70-d0b6-491c-9e9e-f5ed77179ce4.FOP 1" realizes "a16d1d72-9db2-4554-af57-411b29793904.FOP 1"
trace "be9b5663-23ea-4413-abeb-3d5df84c93b8.FIP 4" realizes "d683afa5-907e-494d-8f35-e10db839feaf.FIP 1"
trace "be9b5663-23ea-4413-abeb-3d5df84c93b8.FOP 1" realizes "d683afa5-907e-494d-8f35-e10db839feaf.FOP 1"
trace "61bf6852-3f28-425d-b8ed-4d38b2b4eebb.FOP 3" realizes "e1808524-dfa9-4883-a809-c44a87712d1b.FOP 3"
trace "27d03649-545b-40b0-8407-721165539005.FIP 1" realizes "e1808524-dfa9-4883-a809-c44a87712d1b.FIP 1"
trace "27d03649-545b-40b0-8407-721165539005.FIP 3" realizes "e1808524-dfa9-4883-a809-c44a87712d1b.FIP 3"
trace "27d03649-545b-40b0-8407-721165539005.FIP 2" realizes "e1808524-dfa9-4883-a809-c44a87712d1b.FIP 2"
trace "27d03649-545b-40b0-8407-721165539005.FOP 4" realizes "e1808524-dfa9-4883-a809-c44a87712d1b.FOP 4"
trace "5a58fd72-1b57-41dc-8b7b-b0b8cf577236.FIP 1" realizes "b7244d6a-08ab-4455-bbd9-aa5f1bbc9b5b.FIP 1"
trace "5a58fd72-1b57-41dc-8b7b-b0b8cf577236.FIP 2" realizes "b7244d6a-08ab-4455-bbd9-aa5f1bbc9b5b.FIP 2"
trace "910a61f1-3da0-404f-9db5-c226af9a3c47.FIP 1" realizes "0a2099f5-2831-4beb-89ae-996a6f54d032.FIP 7"
trace "910a61f1-3da0-404f-9db5-c226af9a3c47.FIP 3" realizes "e1808524-dfa9-4883-a809-c44a87712d1b.FIP 4"
trace "faf3223f-3f47-4ad6-9c58-353b34e1a30f.FIP 2" realizes "5e221dee-8dc7-4a29-9224-ab53fedb3995.FIP 2"
trace "faf3223f-3f47-4ad6-9c58-353b34e1a30f.FIP 3" realizes "5e221dee-8dc7-4a29-9224-ab53fedb3995.FIP 1"
trace "faf3223f-3f47-4ad6-9c58-353b34e1a30f.FOP 2" realizes "5e221dee-8dc7-4a29-9224-ab53fedb3995.FOP 2"
trace "c3b17f24-ae52-413f-ace7-6baab1287b02.FOP 4" realizes "2c453928-a098-4f8a-a1e6-fc545a2840d1.FOP 4"
trace "2ea2703a-6dd1-4835-aaae-16179864ac1d.FIP1" realizes "2c453928-a098-4f8a-a1e6-fc545a2840d1.FIP1"
trace "5165a225-e662-4b4a-95cd-c94fc5726524.FOP 1" realizes "4dbd5747-9662-4378-88cf-270b69ef3827.FOP 1"
trace "5165a225-e662-4b4a-95cd-c94fc5726524.FOP 2" realizes "4dbd5747-9662-4378-88cf-270b69ef3827.FOP 3"
trace "5165a225-e662-4b4a-95cd-c94fc5726524.FOP 6" realizes "4dbd5747-9662-4378-88cf-270b69ef3827.FOP 2"
trace "5165a225-e662-4b4a-95cd-c94fc5726524.FOP 7" realizes "4dbd5747-9662-4378-88cf-270b69ef3827.FOP 6"
trace "70f1acf3-a32d-4190-b97e-3eb361a559e7.FOP1" realizes "2c453928-a098-4f8a-a1e6-fc545a2840d1.FOP1"
trace "94aebe8b-af13-4a7a-a7ee-8fd10a7fe489.FIP 2" realizes "0a2099f5-2831-4beb-89ae-996a6f54d032.FIP 2"
trace "94aebe8b-af13-4a7a-a7ee-8fd10a7fe489.FIP 1" realizes "0a2099f5-2831-4beb-89ae-996a6f54d032.FIP 1"
trace "94aebe8b-af13-4a7a-a7ee-8fd10a7fe489.FIP 3" realizes "0a2099f5-2831-4beb-89ae-996a6f54d032.FIP 4"
trace "94aebe8b-af13-4a7a-a7ee-8fd10a7fe489.FIP 4" realizes "0a2099f5-2831-4beb-89ae-996a6f54d032.FIP 3"
trace "94aebe8b-af13-4a7a-a7ee-8fd10a7fe489.FOP 1" realizes "0a2099f5-2831-4beb-89ae-996a6f54d032.FOP 1"
trace "94aebe8b-af13-4a7a-a7ee-8fd10a7fe489.FOP 2" realizes "0a2099f5-2831-4beb-89ae-996a6f54d032.FOP 2"
trace "94aebe8b-af13-4a7a-a7ee-8fd10a7fe489.FOP 4" realizes "0a2099f5-2831-4beb-89ae-996a6f54d032.FOP 3"
trace "7b604e03-85be-4d93-8035-8a84ef8086db.FOP 1" realizes "e1808524-dfa9-4883-a809-c44a87712d1b.FOP 1"
trace "3136218c-bf14-437d-99c9-6593352eead9.FOP 3" realizes "d8b8c1bd-a2ea-47fd-a7a9-0239408db1a5.FOP 3"
trace "b4623d17-6e62-411d-839c-2d768664a920.FIP 3" realizes "d8b8c1bd-a2ea-47fd-a7a9-0239408db1a5.FIP 3"
trace "2632b523-b577-48c5-afba-3eb4ccde3254.FOP1" realizes "e1808524-dfa9-4883-a809-c44a87712d1b.FOP 2"
trace "a90e98e8-741e-478a-9e14-af7a50ee2525.FIP1" realizes "6e577fe5-ebd7-409e-b5de-b876fd689851.FIP1"
trace "a90e98e8-741e-478a-9e14-af7a50ee2525.FOP1" realizes "6e577fe5-ebd7-409e-b5de-b876fd689851.FOP1"
trace "7069fd31-6eb0-49fa-ae52-0b78307c2748.FOP 3" realizes "06b8dd9b-2199-4ad1-a9b2-229d1de1bb48.FOP 3"
trace "7069fd31-6eb0-49fa-ae52-0b78307c2748.FOP 4" realizes "e1808524-dfa9-4883-a809-c44a87712d1b.FOP 6"
trace "b55db2ae-a407-4c31-8093-75f0841094eb.FIP 1" realizes "cfde8bb1-4df9-4573-a516-41df984ac753.FIP2"
trace "0d727549-f01f-4cc6-ae27-5f25220ece06.FOP2" realizes "d683afa5-907e-494d-8f35-e10db839feaf.FOP 2"
trace "724d6db9-9175-450b-8f05-023fcffec79c" realizes "e0a46739-7e50-4ea6-b198-0e5efb9ce68f"
trace "bcb08500-a828-43b0-ac8b-ff4c7cb3b6b4.FIP 1" realizes "4dbd5747-9662-4378-88cf-270b69ef3827.FIP 2"
trace "bcb08500-a828-43b0-ac8b-ff4c7cb3b6b4.FIP 2" realizes "4dbd5747-9662-4378-88cf-270b69ef3827.FIP 1"
trace "bcb08500-a828-43b0-ac8b-ff4c7cb3b6b4.FIP 3" realizes "4dbd5747-9662-4378-88cf-270b69ef3827.FIP 3"
trace "18da001e-dd8b-4041-ba4c-eb2211c624c4.FIP 1" realizes "e04025f9-6b52-4377-bac1-78d0bf2f8fed.FIP 1"
trace "63136eb0-622b-43cc-b4da-55b86ae281c1.FIP1" realizes "65787ee3-ab55-4200-91e3-d772a4ee3f28.FIP1"
trace "63136eb0-622b-43cc-b4da-55b86ae281c1.FOP1" realizes "65787ee3-ab55-4200-91e3-d772a4ee3f28.FOP1"
trace "295db719-853a-4cb6-9bbe-2e66b5ae9ace.FIP3" realizes "1bc70a69-c6ad-4203-abb9-88b644ac0b95.FIP3"
trace "295db719-853a-4cb6-9bbe-2e66b5ae9ace.FIP2" realizes "1bc70a69-c6ad-4203-abb9-88b644ac0b95.FIP2"
trace "aeafffae-c0e8-45a3-9ad1-105825574ced.FIP1" realizes "880520c2-6bd9-4481-b9da-6ccee7b9d665.FIP1"
trace "395ee0ab-cb3d-48e8-a411-09a98053292f" realizes "181a678c-dca9-46c1-9d18-b5a0c457c0de"
trace "37c0f280-8012-4736-82df-4e9114fc61d4.FIP 2" realizes "cfde8bb1-4df9-4573-a516-41df984ac753.FIP1"
trace "94f479d0-c672-4a9c-bfbb-1fb5e01a73ee" realizes "4890eae8-7a62-4d7b-bf8a-1590a199eebf"
trace "c5e641ab-a062-4856-87ae-e675061c3eb5.FOP1" realizes "6b9e59a9-89f2-497e-a348-3aef80689e61.FOP1"
trace "c6ed6258-be4c-4784-a6e4-3e95f9f22dc7.FOP 5" realizes "8f279b07-26aa-46f6-8c9a-2a132f0b1177.FOP 5"
trace "a0ceff37-72a6-4da7-a646-89425a748235" realizes "c8b78c78-5b11-4fc0-87b7-3ca84622efea"
trace "9d3d6fe7-16df-4382-9d49-492e4cd2b73f.FOP 1" realizes "5e0fff47-a1a5-4a13-867c-18348bd894af.FOP 1"
trace "d5254d45-3946-431f-8162-a28a3f2263a7.FOP 1" realizes "4d0046b4-4865-438d-b09b-fd95671db09b.FOP 1"
trace "d459ccb1-cad9-4756-bb0c-b151c60461e7" realizes "d475ed7d-4644-4461-97d4-b52772c9b425"
trace "c385bac1-ff71-4f7c-b203-40e7779a63f8" realizes "49328560-baa6-492c-ad89-8166ce70fdad"
trace "7b3d0795-e518-4b28-9804-cbb023e2640b" realizes "149f7acb-773d-4960-93a6-df36efee81d0"
trace "1070ab6d-6e51-4aa9-8950-c3d49c0ca727" realizes "56fc8021-e30e-44ae-9d5e-f776874ea5a9"
trace "df26fc8b-2837-40a0-ac8a-96ad5c39f9ac" realizes "4125bbcd-f124-4bd5-9f40-aa73c538a3e4"
trace "127898d1-536b-4fbe-a630-5f0109d15cea" realizes "0048b9e1-315c-4025-a694-d91e75250c19"
trace "6fd71d7f-b493-4678-87d1-515eccf54213" realizes "8aa8e222-d82c-4132-ac6d-9b5f8691a89e"
trace "27751fcd-c868-4b0b-8de7-2c819e912edf" realizes "5e30c0be-9342-477c-82a1-c1d1ba0682fa"
trace "bbabfba6-7fbf-47e0-a5db-d14b1c72f798" realizes "323c5735-2b57-4f13-936f-edabf89c3785"
trace "ff081aeb-7c3e-4778-b8ab-6d1c3718aab9" realizes "018a09b4-bb7a-4714-8814-613e3af68bfe"
trace "8e23a789-19bb-4c39-85ab-ada245047400" realizes "13425779-3b59-4d68-8734-cd9717b69d29"
trace "98cb1109-6bb4-4592-bf25-99bdc93db83a" realizes "4411a2bf-9f4e-40f3-b66e-c82b89483b66"
trace "c2d17c2a-ff11-417e-989f-48b54d4388a4" realizes "6dda546d-7b7f-4ff4-939c-d9c8e441a3a8"
trace "31bc5010-decb-4d62-96eb-1628496778de" realizes "6dda546d-7b7f-4ff4-939c-d9c8e441a3a8"
trace "eb002c8e-031e-4129-8a22-a4f175353b9e" realizes "6dda546d-7b7f-4ff4-939c-d9c8e441a3a8"
trace "7b1d7bc7-ec98-4f52-a99c-418a36980d63" realizes "2d2e8936-7dc3-40da-bb09-6d3f2ce71921"
trace "89c75525-66d8-4da4-a704-9641017c30d1" realizes "e7ca817a-2207-4beb-b7ee-98a679f3e0fb"
trace "544601f7-fbc3-4077-918c-b9199e7452e1" realizes "19cd3b56-9087-4ee6-bf0b-ebd2f730a3e3"
trace "8c966c78-51fb-4436-b36e-cc857783924d" realizes "e3c0b2f5-ec50-4dba-9808-04d6179ad01e"
trace "223cf5ec-0222-4e32-9646-71bbc85cdd70" realizes "5d23ddc8-b208-4db0-8b20-680568afdc92"
trace "654aacca-49b2-49fc-949c-ed6a871c6e42" realizes "15c3b494-a0b8-4aca-b734-01978120cbd4"
trace "77bbd9a8-d804-4b57-aab6-77bd6c5add42" realizes "af919966-017f-44fa-856a-f680d604ded8"
trace "becb1a69-c67a-40b4-8d9e-2ff2c0f6761e" realizes "0ce48a3f-6a6d-498c-b41e-280d1bc64928"
trace "3b5f13a1-3d95-4fa5-95e8-4724c2e4a063" realizes "dc5045e3-96c3-4fc1-b144-305a23255b35"
trace "6f60db60-8fc5-427f-9b15-fc4a6373d642" realizes "8dcadf53-92b7-453c-a4a6-3e3780eabb82"
trace "589697fe-d350-452c-9afa-5cdd98027e5e" realizes "5a141688-cd8a-4211-84ad-0d55c85888cd"
trace "68708a1d-448b-483f-96cd-4478cc3ff088" realizes "99249234-8b24-490f-b746-383bfe3087b8"
trace "02a756c6-5f3e-41aa-80ee-b6f7049b959f" realizes "604081d7-e597-4441-b2e9-e002a9cb1ccd"
trace "6265c622-964e-4caa-bf7d-49a2d4b48ad0" realizes "c8bfbb5b-865d-4924-902f-b209dbfb3433"
trace "07d33b69-6881-40bd-9573-0f2d3e790544" realizes "728283a8-142b-4afe-b67c-f2f75cf5a3a3"
trace "f4478adf-d442-4e3b-a3bc-bc108f92957e" realizes "7531be55-55d9-44a9-8077-727417cf4f0e"
trace "f1f7e78f-7745-4687-a89b-229520c41a8d" realizes "13425779-3b59-4d68-8734-cd9717b69d29"
trace "f1f7e78f-7745-4687-a89b-229520c41a8d" realizes "323c5735-2b57-4f13-936f-edabf89c3785"
trace "f1f7e78f-7745-4687-a89b-229520c41a8d" realizes "018a09b4-bb7a-4714-8814-613e3af68bfe"
trace "b78c85ec-d3c9-43f0-a006-aa5340599509" realizes "21e787ec-7763-4896-a0c6-421367c3561a"
trace "42c74c0c-9903-4d3c-9372-bdbaf742257b" realizes "fa5c887c-feac-438f-ac22-2bd908b30483"
trace "a565c017-0a94-4788-baf4-835bf3d03205" realizes "613d97e8-7adb-4f2b-a3de-359c73087bff"
trace "0e1ff1e8-00c9-4cb9-8d96-7cdc2ebcf735" realizes "29fdf266-fe43-4715-b919-cb4ce94b35d7"
trace "86d0f636-e27f-436d-9e58-0ce047bd6cdd" realizes "7f88eaab-0c1f-456b-b32f-f65c5bf00454"
trace "0ad613f5-c36e-4cb8-b9f9-ce38b8078929" realizes "c9afe3ac-0a52-450a-b97d-741e4b03ad39"
trace "6e91454a-ca43-4d1e-b8f8-22435ec9c59a" realizes "c4d953c9-05e2-42ca-b110-7014f768d808"
trace "1f6c53f8-243d-4981-b1f7-37c597cde3d3" realizes "7531be55-55d9-44a9-8077-727417cf4f0e"
trace "91642f3d-3687-43e4-946c-71541aa590c5" realizes "0ce48a3f-6a6d-498c-b41e-280d1bc64928"
trace "7677133e-57ee-4827-8cbf-73ee1f55a5a4" realizes "136ac12c-9e93-43b9-bc3d-4253ea48233a"
trace "cdc4500f-0b03-4016-97eb-6094a12202aa" realizes "fa5c887c-feac-438f-ac22-2bd908b30483"
trace "6d34ef5b-feba-4360-8812-382fe34c0bd3" realizes "613d97e8-7adb-4f2b-a3de-359c73087bff"
trace "66ca912b-4dc1-4616-9a24-c24fef5ac11e" realizes "8aa8e222-d82c-4132-ac6d-9b5f8691a89e"
trace "d8ae3120-7282-4314-bf35-41dd4c2e82f6" realizes "883d204e-69ff-40e9-80e4-e4d5845bb2d2"
trace "8ab8e6a5-2c1f-4046-8aab-b44dd550b833" realizes "149f7acb-773d-4960-93a6-df36efee81d0"
trace "929f4bd6-ff1c-43d4-8361-abd7382f5d91" realizes "49328560-baa6-492c-ad89-8166ce70fdad"
trace "256a5004-f2cc-4c22-a4d4-715d4a775764" realizes "cc3abddf-d289-4461-925d-c4d962230954"
trace "6bce3c7b-9f42-439d-adcf-5e47055d355a" realizes "b40bc1cf-4622-41ec-801f-1f37102f7ff9"
trace "8026ed14-01f5-42c4-8c69-d36afeee1075" realizes "0bc7709d-62f9-4a1e-af23-b5a8a78ac19c"
trace "0e7dae41-7386-402e-9e33-213a1bae69a2" realizes "8016239b-3648-4d04-85d4-76305d740490"
trace "2c650bc2-d254-4026-b5cf-c2a001f289bd" realizes "883d204e-69ff-40e9-80e4-e4d5845bb2d2"
trace "56f511e4-5d06-42d3-b96e-d5a2522f6bed" realizes "dd56a4c7-8304-4471-bf3f-9ffddaccf544"
trace "8fe69f90-7472-4e05-ab57-1e0fa4e511a4" realizes "adb3033a-d058-4995-98e9-60aaf85f4328"
trace "849047d6-233a-4f85-bf60-8df1f145cda9" realizes "f5c2a034-4749-41c2-a77e-6ecaae28f5d0"
trace "3def4748-6a03-4e18-8004-0cacb9f3b457" realizes "7d375159-5cdb-4959-8e5c-98f43941240c"
trace "5eb59259-7021-4c20-805c-9115a89867a0" realizes "2c6157aa-a353-4db7-9f05-05bb1db08215"
trace "24ca4fcd-9fe9-404c-8ce8-57dd25b038dc" realizes "e00b45ce-330b-4b19-ac67-07095a77f8c0"
trace "9601e912-9538-4dde-9c4e-c631179e763c" realizes "5ecacd76-5aa1-4a9f-91f5-e00bad983206"
trace "e57ba3b5-fd92-4bf6-8b4d-1c7df1602a20" realizes "0293672b-2202-4438-8ad4-86af57df02c3"
trace "7cde2018-8a87-450f-934a-bca3ea150873" realizes "b69dc23e-97e5-4f8f-b4b4-704fcd851d9e"
trace "1f37cd9c-1aa0-4698-95f9-c87437af0914" realizes "10a21953-49e6-4179-8ce8-aafc125e9540"
trace "0378eba1-fb7b-4b4d-b39d-7a14ce4e5f4e.FIP 1" realizes "be9b5663-23ea-4413-abeb-3d5df84c93b8.FIP 1"
trace "0378eba1-fb7b-4b4d-b39d-7a14ce4e5f4e.FIP 2" realizes "be9b5663-23ea-4413-abeb-3d5df84c93b8.FIP 2"
trace "0378eba1-fb7b-4b4d-b39d-7a14ce4e5f4e.FIP 3" realizes "be9b5663-23ea-4413-abeb-3d5df84c93b8.FIP 3"
trace "0378eba1-fb7b-4b4d-b39d-7a14ce4e5f4e.FIP 4" realizes "be9b5663-23ea-4413-abeb-3d5df84c93b8.FIP 4"
trace "0378eba1-fb7b-4b4d-b39d-7a14ce4e5f4e.FOP 1" realizes "be9b5663-23ea-4413-abeb-3d5df84c93b8.FOP 1"
trace "0378eba1-fb7b-4b4d-b39d-7a14ce4e5f4e.FOP 2" realizes "be9b5663-23ea-4413-abeb-3d5df84c93b8.FOP 2"
trace "0378eba1-fb7b-4b4d-b39d-7a14ce4e5f4e.FOP 3" realizes "be9b5663-23ea-4413-abeb-3d5df84c93b8.FOP 3"
trace "0378eba1-fb7b-4b4d-b39d-7a14ce4e5f4e.FOP 4" realizes "be9b5663-23ea-4413-abeb-3d5df84c93b8.FOP 4"
trace "0378eba1-fb7b-4b4d-b39d-7a14ce4e5f4e.FOP 5" realizes "be9b5663-23ea-4413-abeb-3d5df84c93b8.FOP 5"
trace "98960be5-3e8f-4743-ab70-805645472512.FIP 2" realizes "61bf6852-3f28-425d-b8ed-4d38b2b4eebb.FIP 2"
trace "d53e6dd4-3926-4b25-80d1-14553a472f9e.FIP 1" realizes "27d03649-545b-40b0-8407-721165539005.FIP 1"
trace "d53e6dd4-3926-4b25-80d1-14553a472f9e.FIP 3" realizes "27d03649-545b-40b0-8407-721165539005.FIP 3"
trace "d53e6dd4-3926-4b25-80d1-14553a472f9e.FIP 2" realizes "27d03649-545b-40b0-8407-721165539005.FIP 2"
trace "d53e6dd4-3926-4b25-80d1-14553a472f9e.FOP 1" realizes "27d03649-545b-40b0-8407-721165539005.FOP 1"
trace "d53e6dd4-3926-4b25-80d1-14553a472f9e.FOP 5" realizes "27d03649-545b-40b0-8407-721165539005.FOP 3"
trace "d53e6dd4-3926-4b25-80d1-14553a472f9e.FOP 4" realizes "27d03649-545b-40b0-8407-721165539005.FOP 4"
trace "d53e6dd4-3926-4b25-80d1-14553a472f9e.FOP 2" realizes "27d03649-545b-40b0-8407-721165539005.FOP 2"
trace "2dfb0a52-7568-4903-9c78-1e462848f868.FOP 1" realizes "61bf6852-3f28-425d-b8ed-4d38b2b4eebb.FOP 1"
trace "2dfb0a52-7568-4903-9c78-1e462848f868.FOP 2" realizes "61bf6852-3f28-425d-b8ed-4d38b2b4eebb.FOP 2"
trace "241dcfae-1880-45f9-bf2c-32b0cbe736b7.FIP 1" realizes "910a61f1-3da0-404f-9db5-c226af9a3c47.FIP 3"
trace "241dcfae-1880-45f9-bf2c-32b0cbe736b7.FIP 2" realizes "910a61f1-3da0-404f-9db5-c226af9a3c47.FIP 1"
trace "241dcfae-1880-45f9-bf2c-32b0cbe736b7.FIP 3" realizes "910a61f1-3da0-404f-9db5-c226af9a3c47.FIP 2"
trace "241dcfae-1880-45f9-bf2c-32b0cbe736b7.FOP 1" realizes "910a61f1-3da0-404f-9db5-c226af9a3c47.FOP 1"
trace "ac5be58c-aa92-4247-8c8f-9867ddbd6289.FIP 1" realizes "faf3223f-3f47-4ad6-9c58-353b34e1a30f.FIP 1"
trace "ac5be58c-aa92-4247-8c8f-9867ddbd6289.FIP 2" realizes "faf3223f-3f47-4ad6-9c58-353b34e1a30f.FIP 2"
trace "ac5be58c-aa92-4247-8c8f-9867ddbd6289.FIP 3" realizes "faf3223f-3f47-4ad6-9c58-353b34e1a30f.FIP 3"
trace "ac5be58c-aa92-4247-8c8f-9867ddbd6289.FOP 2" realizes "faf3223f-3f47-4ad6-9c58-353b34e1a30f.FOP 2"
trace "dcbf11ee-150a-4afa-8e06-3f0ab9aee27a.FIP 1" realizes "651179c3-78b6-4a6f-a174-74405b81b36a.FIP 1"
trace "dcbf11ee-150a-4afa-8e06-3f0ab9aee27a.FIP 2" realizes "651179c3-78b6-4a6f-a174-74405b81b36a.FIP 2"
trace "dcbf11ee-150a-4afa-8e06-3f0ab9aee27a.FIP 3" realizes "651179c3-78b6-4a6f-a174-74405b81b36a.FIP 3"
trace "dcbf11ee-150a-4afa-8e06-3f0ab9aee27a.FIP 4" realizes "651179c3-78b6-4a6f-a174-74405b81b36a.FIP 4"
trace "dcbf11ee-150a-4afa-8e06-3f0ab9aee27a.FIP 5" realizes "651179c3-78b6-4a6f-a174-74405b81b36a.FIP 5"
trace "dcbf11ee-150a-4afa-8e06-3f0ab9aee27a.FIP 6" realizes "651179c3-78b6-4a6f-a174-74405b81b36a.FIP 6"
trace "dcbf11ee-150a-4afa-8e06-3f0ab9aee27a.FIP 7" realizes "651179c3-78b6-4a6f-a174-74405b81b36a.FIP 7"
trace "dcbf11ee-150a-4afa-8e06-3f0ab9aee27a.FOP 1" realizes "651179c3-78b6-4a6f-a174-74405b81b36a.FOP 1"
trace "dcbf11ee-150a-4afa-8e06-3f0ab9aee27a.FOP 2" realizes "651179c3-78b6-4a6f-a174-74405b81b36a.FOP 2"
trace "dcbf11ee-150a-4afa-8e06-3f0ab9aee27a.FOP 3" realizes "651179c3-78b6-4a6f-a174-74405b81b36a.FOP 3"
trace "dcbf11ee-150a-4afa-8e06-3f0ab9aee27a.FOP 4" realizes "651179c3-78b6-4a6f-a174-74405b81b36a.FOP 4"
trace "dcbf11ee-150a-4afa-8e06-3f0ab9aee27a.FOP 5" realizes "651179c3-78b6-4a6f-a174-74405b81b36a.FOP 5"
trace "8cc1e6f5-dbe2-4629-8a43-b60eef771a8f.FIP1" realizes "c3b17f24-ae52-413f-ace7-6baab1287b02.FIP1"
trace "8cc1e6f5-dbe2-4629-8a43-b60eef771a8f.FIP 2" realizes "c3b17f24-ae52-413f-ace7-6baab1287b02.FIP 2"
trace "8cc1e6f5-dbe2-4629-8a43-b60eef771a8f.FIP 4" realizes "c3b17f24-ae52-413f-ace7-6baab1287b02.FIP 4"
trace "8cc1e6f5-dbe2-4629-8a43-b60eef771a8f.FOP 1" realizes "c3b17f24-ae52-413f-ace7-6baab1287b02.FOP 1"
trace "8cc1e6f5-dbe2-4629-8a43-b60eef771a8f.FOP 4" realizes "c3b17f24-ae52-413f-ace7-6baab1287b02.FOP 4"
trace "4c361e00-d6d0-4acf-a523-1de11acf7797.FIP 1" realizes "7b604e03-85be-4d93-8035-8a84ef8086db.FIP 1"
trace "8daae3ab-944b-4ade-9896-0d51c62ef3e8.FOP 3" realizes "3136218c-bf14-437d-99c9-6593352eead9.FOP 3"
trace "3f69f8bd-704d-438b-8f05-fc7d56363811.FIP 3" realizes "b4623d17-6e62-411d-839c-2d768664a920.FIP 3"
trace "3f69f8bd-704d-438b-8f05-fc7d56363811.FOP1" realizes "b4623d17-6e62-411d-839c-2d768664a920.FOP1"
trace "3f69f8bd-704d-438b-8f05-fc7d56363811.FOP 2" realizes "b4623d17-6e62-411d-839c-2d768664a920.FOP 2"
trace "8835c15a-0eb0-4d20-83d2-a15f8c0bd005.FIP 1" realizes "3e9ccc30-596e-40e6-8941-64838248dcee.FIP 1"
trace "8835c15a-0eb0-4d20-83d2-a15f8c0bd005.FIP 2" realizes "3e9ccc30-596e-40e6-8941-64838248dcee.FIP 2"
trace "8835c15a-0eb0-4d20-83d2-a15f8c0bd005.FOP 3" realizes "3e9ccc30-596e-40e6-8941-64838248dcee.FOP 3"
trace "ecc7d466-f9cb-4be1-a099-b8449146cf18.FIP 2" realizes "7b604e03-85be-4d93-8035-8a84ef8086db.FIP 2"
trace "068dc2db-3e29-421d-84be-8444ce8acc9c.FIP 1" realizes "4ff700ef-4f6b-4151-917b-e59064e5a225.FIP 1"
trace "068dc2db-3e29-421d-84be-8444ce8acc9c.FOP 1" realizes "4ff700ef-4f6b-4151-917b-e59064e5a225.FOP 1"
trace "83b4863a-7795-4911-aeef-2938443cce3d.FIP 1" realizes "70f1acf3-a32d-4190-b97e-3eb361a559e7.FIP 1"
trace "83b4863a-7795-4911-aeef-2938443cce3d.FIP 2" realizes "70f1acf3-a32d-4190-b97e-3eb361a559e7.FIP 2"
trace "83b4863a-7795-4911-aeef-2938443cce3d.FOP1" realizes "70f1acf3-a32d-4190-b97e-3eb361a559e7.FOP1"
trace "83b4863a-7795-4911-aeef-2938443cce3d.FOP 2" realizes "70f1acf3-a32d-4190-b97e-3eb361a559e7.FOP 2"
trace "c7edaab3-11bc-4409-b300-13c4b4192742.FIP 2" realizes "94aebe8b-af13-4a7a-a7ee-8fd10a7fe489.FIP 2"
trace "c7edaab3-11bc-4409-b300-13c4b4192742.FIP 1" realizes "94aebe8b-af13-4a7a-a7ee-8fd10a7fe489.FIP 1"
trace "c7edaab3-11bc-4409-b300-13c4b4192742.FIP 3" realizes "94aebe8b-af13-4a7a-a7ee-8fd10a7fe489.FIP 3"
trace "c7edaab3-11bc-4409-b300-13c4b4192742.FIP 4" realizes "94aebe8b-af13-4a7a-a7ee-8fd10a7fe489.FIP 4"
trace "c7edaab3-11bc-4409-b300-13c4b4192742.FOP 1" realizes "94aebe8b-af13-4a7a-a7ee-8fd10a7fe489.FOP 1"
trace "c7edaab3-11bc-4409-b300-13c4b4192742.FOP 2" realizes "94aebe8b-af13-4a7a-a7ee-8fd10a7fe489.FOP 2"
trace "c7edaab3-11bc-4409-b300-13c4b4192742.FOP 3" realizes "94aebe8b-af13-4a7a-a7ee-8fd10a7fe489.FOP 4"
trace "bd383ff2-258a-41dd-973f-07f7aa30da7a.FOP 1" realizes "7b604e03-85be-4d93-8035-8a84ef8086db.FOP 1"
trace "bd383ff2-258a-41dd-973f-07f7aa30da7a.FOP 2" realizes "2632b523-b577-48c5-afba-3eb4ccde3254.FOP1"
trace "9155e7ae-b4c3-4f06-b8b2-0ec90045ac8e.FIP 1" realizes "7b604e03-85be-4d93-8035-8a84ef8086db.FIP 3"
trace "9d76fefa-2f89-4272-8b1d-785d05601ce6.FIP 1" realizes "2ea2703a-6dd1-4835-aaae-16179864ac1d.FIP 1"
trace "9d76fefa-2f89-4272-8b1d-785d05601ce6.FIP1" realizes "2ea2703a-6dd1-4835-aaae-16179864ac1d.FIP1"
trace "9d76fefa-2f89-4272-8b1d-785d05601ce6.FOP 2" realizes "2ea2703a-6dd1-4835-aaae-16179864ac1d.FOP 2"
trace "9d76fefa-2f89-4272-8b1d-785d05601ce6.FOP1" realizes "2ea2703a-6dd1-4835-aaae-16179864ac1d.FOP1"
trace "9d76fefa-2f89-4272-8b1d-785d05601ce6.FOP 1" realizes "2ea2703a-6dd1-4835-aaae-16179864ac1d.FOP 1"
trace "0c8009a9-0524-4445-a9c8-f0295467f62a.FIP 1" realizes "5165a225-e662-4b4a-95cd-c94fc5726524.FIP 1"
trace "0c8009a9-0524-4445-a9c8-f0295467f62a.FIP 2" realizes "5165a225-e662-4b4a-95cd-c94fc5726524.FIP 2"
trace "0c8009a9-0524-4445-a9c8-f0295467f62a.FIP 3" realizes "5165a225-e662-4b4a-95cd-c94fc5726524.FIP 3"
trace "0c8009a9-0524-4445-a9c8-f0295467f62a.FIP 4" realizes "5165a225-e662-4b4a-95cd-c94fc5726524.FIP 4"
trace "0c8009a9-0524-4445-a9c8-f0295467f62a.FIP 5" realizes "5165a225-e662-4b4a-95cd-c94fc5726524.FIP 5"
trace "0c8009a9-0524-4445-a9c8-f0295467f62a.FOP 1" realizes "5165a225-e662-4b4a-95cd-c94fc5726524.FOP 1"
trace "0c8009a9-0524-4445-a9c8-f0295467f62a.FOP 2" realizes "5165a225-e662-4b4a-95cd-c94fc5726524.FOP 2"
trace "0c8009a9-0524-4445-a9c8-f0295467f62a.FOP 3" realizes "5165a225-e662-4b4a-95cd-c94fc5726524.FOP 3"
trace "0c8009a9-0524-4445-a9c8-f0295467f62a.FOP 4" realizes "5165a225-e662-4b4a-95cd-c94fc5726524.FOP 4"
trace "0c8009a9-0524-4445-a9c8-f0295467f62a.FOP 5" realizes "5165a225-e662-4b4a-95cd-c94fc5726524.FOP 5"
trace "0c8009a9-0524-4445-a9c8-f0295467f62a.FOP 6" realizes "5165a225-e662-4b4a-95cd-c94fc5726524.FOP 6"
trace "0c8009a9-0524-4445-a9c8-f0295467f62a.FOP 7" realizes "5165a225-e662-4b4a-95cd-c94fc5726524.FOP 7"
trace "737aebcf-1f00-40ac-8e03-df8bc6596159.FIP 2" realizes "7069fd31-6eb0-49fa-ae52-0b78307c2748.FIP 2"
trace "737aebcf-1f00-40ac-8e03-df8bc6596159.FIP 4" realizes "7069fd31-6eb0-49fa-ae52-0b78307c2748.FIP 4"
trace "737aebcf-1f00-40ac-8e03-df8bc6596159.FIP 3" realizes "7069fd31-6eb0-49fa-ae52-0b78307c2748.FIP 3"
trace "737aebcf-1f00-40ac-8e03-df8bc6596159.FOP 3" realizes "7069fd31-6eb0-49fa-ae52-0b78307c2748.FOP 3"
trace "737aebcf-1f00-40ac-8e03-df8bc6596159.FOP 4" realizes "7069fd31-6eb0-49fa-ae52-0b78307c2748.FOP 4"
trace "e8af73d3-bacb-4222-b387-f6b0bd4d13a1.FIP 4" realizes "b55db2ae-a407-4c31-8093-75f0841094eb.FIP 4"
trace "e8af73d3-bacb-4222-b387-f6b0bd4d13a1.FIP 1" realizes "b55db2ae-a407-4c31-8093-75f0841094eb.FIP 1"
trace "e8af73d3-bacb-4222-b387-f6b0bd4d13a1.FOP 1" realizes "b55db2ae-a407-4c31-8093-75f0841094eb.FOP 1"
trace "1cda3b05-ee17-4912-b875-40bc4ddca86b.FIP 1" realizes "0d727549-f01f-4cc6-ae27-5f25220ece06.FIP 1"
trace "1cda3b05-ee17-4912-b875-40bc4ddca86b.FIP 2" realizes "0d727549-f01f-4cc6-ae27-5f25220ece06.FIP 2"
trace "1cda3b05-ee17-4912-b875-40bc4ddca86b.FIP 3" realizes "0d727549-f01f-4cc6-ae27-5f25220ece06.FIP 3"
trace "1cda3b05-ee17-4912-b875-40bc4ddca86b.FOP2" realizes "0d727549-f01f-4cc6-ae27-5f25220ece06.FOP2"
trace "1cda3b05-ee17-4912-b875-40bc4ddca86b.FOP 2" realizes "0d727549-f01f-4cc6-ae27-5f25220ece06.FOP 2"
trace "9b798c7d-2d5a-43f9-867d-b26c3f4c1337.FIP 1" realizes "1aa42ce5-1c2c-4473-9840-7d20a60c377e.FIP 1"
trace "9b798c7d-2d5a-43f9-867d-b26c3f4c1337.FIP 2" realizes "1aa42ce5-1c2c-4473-9840-7d20a60c377e.FIP 2"
trace "9b798c7d-2d5a-43f9-867d-b26c3f4c1337.FIP 3" realizes "1aa42ce5-1c2c-4473-9840-7d20a60c377e.FIP 3"
trace "9b798c7d-2d5a-43f9-867d-b26c3f4c1337.FIP 4" realizes "1aa42ce5-1c2c-4473-9840-7d20a60c377e.FIP 4"
trace "9b798c7d-2d5a-43f9-867d-b26c3f4c1337.FOP 2" realizes "1aa42ce5-1c2c-4473-9840-7d20a60c377e.FOP 2"
trace "9b798c7d-2d5a-43f9-867d-b26c3f4c1337.FOP 3" realizes "1aa42ce5-1c2c-4473-9840-7d20a60c377e.FOP 3"
trace "9b798c7d-2d5a-43f9-867d-b26c3f4c1337.FOP 4" realizes "1aa42ce5-1c2c-4473-9840-7d20a60c377e.FOP 4"
trace "9b798c7d-2d5a-43f9-867d-b26c3f4c1337.FOP 5" realizes "1aa42ce5-1c2c-4473-9840-7d20a60c377e.FOP 5"
trace "cebc5976-81d6-4718-b1f0-9f8b7687ceeb.FIP1" realizes "a90e98e8-741e-478a-9e14-af7a50ee2525.FIP1"
trace "cebc5976-81d6-4718-b1f0-9f8b7687ceeb.FOP1" realizes "a90e98e8-741e-478a-9e14-af7a50ee2525.FOP1"
trace "cebc5976-81d6-4718-b1f0-9f8b7687ceeb.FOP2" realizes "a90e98e8-741e-478a-9e14-af7a50ee2525.FOP2"
trace "72f0a4da-866e-4136-8ca7-bfc85db05005.FIP 1" realizes "566f8900-7408-42d3-82f8-ba18bf0d923f.FIP 1"
trace "72f0a4da-866e-4136-8ca7-bfc85db05005.FOP 1" realizes "566f8900-7408-42d3-82f8-ba18bf0d923f.FOP 1"
trace "16904687-4018-46cc-a0ec-c415f6d2bc1d.FIP 1" realizes "70d62f70-d0b6-491c-9e9e-f5ed77179ce4.FIP 1"
trace "16904687-4018-46cc-a0ec-c415f6d2bc1d.FOP 1" realizes "70d62f70-d0b6-491c-9e9e-f5ed77179ce4.FOP 1"
trace "2788898f-6daf-43c1-bca8-6ab484ac24c6.FIP 1" realizes "ee920873-7e2d-487f-a51e-76236d6b094e.FIP 1"
trace "b4c36894-10af-4166-9da7-099de70ad0f6.FIP 1" realizes "4b01ac5e-7920-491d-af35-1846163e5ad9.FIP 1"
trace "b4c36894-10af-4166-9da7-099de70ad0f6.FIP 2" realizes "4b01ac5e-7920-491d-af35-1846163e5ad9.FIP 2"
trace "b4c36894-10af-4166-9da7-099de70ad0f6.FOP 1" realizes "4b01ac5e-7920-491d-af35-1846163e5ad9.FOP 1"
trace "b4c36894-10af-4166-9da7-099de70ad0f6.FOP 2" realizes "4b01ac5e-7920-491d-af35-1846163e5ad9.FOP 2"
trace "c13d6970-0ca4-44b4-be4b-64b6f3eebbb4.FIP 1" realizes "1e812d3d-6c35-42c9-80fa-eb64dfc1a3a8.FIP 1"
trace "c13d6970-0ca4-44b4-be4b-64b6f3eebbb4.FIP 2" realizes "1e812d3d-6c35-42c9-80fa-eb64dfc1a3a8.FIP 2"
trace "c13d6970-0ca4-44b4-be4b-64b6f3eebbb4.FOP 1" realizes "1e812d3d-6c35-42c9-80fa-eb64dfc1a3a8.FOP 1"
trace "88a4d6d0-fcf2-4c03-be5e-e6a23c727bf4.FIP 1" realizes "5a58fd72-1b57-41dc-8b7b-b0b8cf577236.FIP 1"
trace "88a4d6d0-fcf2-4c03-be5e-e6a23c727bf4.FIP 2" realizes "5a58fd72-1b57-41dc-8b7b-b0b8cf577236.FIP 2"
trace "88a4d6d0-fcf2-4c03-be5e-e6a23c727bf4.FIP 3" realizes "5a58fd72-1b57-41dc-8b7b-b0b8cf577236.FIP 3"
trace "88a4d6d0-fcf2-4c03-be5e-e6a23c727bf4.FOP 1" realizes "5a58fd72-1b57-41dc-8b7b-b0b8cf577236.FOP 1"
trace "80a8d0f6-ca9e-486f-9ac3-0c524de61281.FOP 3" realizes "39f96e89-8ead-4742-bc1f-6eb0dd33990b.FOP 3"
trace "c91204f5-d5ed-401e-b83d-d6e85d7d887c.FIP 3" realizes "2632b523-b577-48c5-afba-3eb4ccde3254.FIP 3"
trace "28b23885-f94c-44b9-8c2f-a2f52b379842.FIP 1" realizes "2632b523-b577-48c5-afba-3eb4ccde3254.FIP 1"
trace "643033c9-5624-45f8-a1ce-b9bc4ddc2553.FIP 1" realizes "bcb08500-a828-43b0-ac8b-ff4c7cb3b6b4.FIP 1"
trace "643033c9-5624-45f8-a1ce-b9bc4ddc2553.FIP 2" realizes "bcb08500-a828-43b0-ac8b-ff4c7cb3b6b4.FIP 2"
trace "643033c9-5624-45f8-a1ce-b9bc4ddc2553.FIP 3" realizes "bcb08500-a828-43b0-ac8b-ff4c7cb3b6b4.FIP 3"
trace "643033c9-5624-45f8-a1ce-b9bc4ddc2553.FOP 1" realizes "bcb08500-a828-43b0-ac8b-ff4c7cb3b6b4.FOP 1"
trace "643033c9-5624-45f8-a1ce-b9bc4ddc2553.FOP 2" realizes "bcb08500-a828-43b0-ac8b-ff4c7cb3b6b4.FOP 2"
trace "643033c9-5624-45f8-a1ce-b9bc4ddc2553.FOP 3" realizes "bcb08500-a828-43b0-ac8b-ff4c7cb3b6b4.FOP 3"
trace "643033c9-5624-45f8-a1ce-b9bc4ddc2553.FOP 4" realizes "bcb08500-a828-43b0-ac8b-ff4c7cb3b6b4.FOP 4"
trace "643033c9-5624-45f8-a1ce-b9bc4ddc2553.FOP 5" realizes "bcb08500-a828-43b0-ac8b-ff4c7cb3b6b4.FOP 5"
trace "3159e079-92f0-4e22-8cef-53e8f447cd5f.FIP 1" realizes "18da001e-dd8b-4041-ba4c-eb2211c624c4.FIP 1"
trace "3159e079-92f0-4e22-8cef-53e8f447cd5f.FIP 2" realizes "18da001e-dd8b-4041-ba4c-eb2211c624c4.FIP 2"
trace "a3605148-c80e-4635-9790-f068451a1b4a.FIP1" realizes "63136eb0-622b-43cc-b4da-55b86ae281c1.FIP1"
trace "a3605148-c80e-4635-9790-f068451a1b4a.FOP1" realizes "63136eb0-622b-43cc-b4da-55b86ae281c1.FOP1"
trace "a3605148-c80e-4635-9790-f068451a1b4a.FOP 2" realizes "63136eb0-622b-43cc-b4da-55b86ae281c1.FOP 2"
trace "649d218e-6119-46b3-8c9e-377cf40e086d.FIP3" realizes "295db719-853a-4cb6-9bbe-2e66b5ae9ace.FIP3"
trace "649d218e-6119-46b3-8c9e-377cf40e086d.FIP2" realizes "295db719-853a-4cb6-9bbe-2e66b5ae9ace.FIP2"
trace "be579285-807c-46ab-bcab-ae7efb19a8d6.FIP1" realizes "aeafffae-c0e8-45a3-9ad1-105825574ced.FIP1"
trace "292159f9-5917-4c0c-9ea3-2800431b3c48.FIP 1" realizes "37c0f280-8012-4736-82df-4e9114fc61d4.FIP 1"
trace "292159f9-5917-4c0c-9ea3-2800431b3c48.FIP 2" realizes "37c0f280-8012-4736-82df-4e9114fc61d4.FIP 2"
trace "292159f9-5917-4c0c-9ea3-2800431b3c48.FIP 3" realizes "37c0f280-8012-4736-82df-4e9114fc61d4.FIP 3"
trace "292159f9-5917-4c0c-9ea3-2800431b3c48.FOP 1" realizes "37c0f280-8012-4736-82df-4e9114fc61d4.FOP 1"
trace "292159f9-5917-4c0c-9ea3-2800431b3c48.FOP 2" realizes "37c0f280-8012-4736-82df-4e9114fc61d4.FOP 2"
trace "292159f9-5917-4c0c-9ea3-2800431b3c48.FOP 3" realizes "37c0f280-8012-4736-82df-4e9114fc61d4.FOP 3"
trace "292159f9-5917-4c0c-9ea3-2800431b3c48.FOP 4" realizes "37c0f280-8012-4736-82df-4e9114fc61d4.FOP 4"
trace "9c247f2d-ed9b-43e8-921a-d1fdbb331ab1.FOP1" realizes "c5e641ab-a062-4856-87ae-e675061c3eb5.FOP1"
trace "4c237128-0cf3-4074-8af4-5e63e4994e02.FOP 1" realizes "4d1c1e5c-4c96-41fc-a4a5-4bd74085f320.FOP 1"
trace "820d4407-7206-4075-91fc-d76ce083056e.FOP 5" realizes "c6ed6258-be4c-4784-a6e4-3e95f9f22dc7.FOP 5"
trace "75e214e4-74f8-4c44-97f9-f328b8ac3f97.FOP 1" realizes "e01b39c5-f018-4d9c-af36-c83f1b9a8a81.FOP 1"
trace "a8b9e953-b751-4991-b90e-e3ec251f65f0.FOP 1" realizes "9d3d6fe7-16df-4382-9d49-492e4cd2b73f.FOP 1"
trace "eab9aed9-2576-4d7b-ad86-021e79aac4e1.FOP 1" realizes "d5254d45-3946-431f-8162-a28a3f2263a7.FOP 1"
trace "21b77a35-529e-4bce-98eb-8c6e8e2fc01a" realizes "1070ab6d-6e51-4aa9-8950-c3d49c0ca727"
trace "f0fb96ca-05e7-4119-99ff-b08e8184206e" realizes "df26fc8b-2837-40a0-ac8a-96ad5c39f9ac"
trace "588d634d-b2c8-4632-8276-000f5b97dace" realizes "127898d1-536b-4fbe-a630-5f0109d15cea"
trace "92d7c48e-f677-4bab-9bdb-ee7bb1eb7a28" realizes "4d67efdd-6484-4d84-a242-6b0210eec6f7"
trace "05c4227e-c46c-4a4b-b9e9-9b822a5ec038" realizes "4bba771e-a9df-467f-acde-0929d06e0d95"
trace "bc4ac210-f2e5-4332-b548-69f52ec5ff64" realizes "99801e2d-2ff3-4e61-bd8b-5fb26601bf96"
trace "7f7b6497-509f-416f-a714-a2e3a6065e76" realizes "2eb58ffb-766e-450d-b436-5d1228203493"
trace "4dc4747f-d2a3-4f50-b290-790bf206ea96" realizes "472103e7-6717-4c09-96dc-a9b4f4b3f221"
trace "f9231b85-ee09-49bc-b407-dd66190a7b82" realizes "f0ae5ac4-8841-4866-b3db-7377f0891da6"
trace "303c4e84-7d9e-4c13-96d3-c898826da1b9" realizes "6fd71d7f-b493-4678-87d1-515eccf54213"
trace "664c04f7-85af-4aef-a7a7-cb4f31832495" realizes "cfbfc6da-1898-4a5d-8a93-84865d97a1b3"
trace "5749f82b-2323-4e33-9fa7-3253d6fdc1ee" realizes "845007fe-fe45-46ec-a109-47742ac52529"
trace "86900f67-a76d-4996-91ce-e94dc33333be" realizes "223cf5ec-0222-4e32-9646-71bbc85cdd70"
trace "67a82bc3-0c6a-457b-8558-333cbb724512" realizes "816d9b3e-7345-4b2b-b67f-1628d8034357"
trace "70ec2604-a2d3-48c3-9594-a162d2ee8182" realizes "7738da4d-7ccc-4eb6-973b-c28c3319d97a"
trace "b4e7ccf9-a718-439f-9ac3-56494cd432d2" realizes "c111592c-86f1-43ab-9979-eb6dc669c9a4"
trace "7739f8e7-2b55-4805-847e-ecdba2a46b46" realizes "27751fcd-c868-4b0b-8de7-2c819e912edf"
trace "7611e13a-7585-4400-9a0f-58adbe6c9fed" realizes "bbabfba6-7fbf-47e0-a5db-d14b1c72f798"
trace "d1520556-1979-4d95-843d-493272c4a36c" realizes "ff081aeb-7c3e-4778-b8ab-6d1c3718aab9"
trace "4ab2fec4-58d0-4f7b-a219-0f8ff8d5ee7a" realizes "8e23a789-19bb-4c39-85ab-ada245047400"
trace "21b38194-329a-4371-82cd-5d6e8085d550" realizes "4aee9cf0-9b14-436e-86ad-5a11d4866e40"
trace "243d3251-c6e2-4973-9810-3dd8c0c39aae" realizes "793b6f83-aadc-4b2e-804d-26190aab3e1d"
trace "ee4691b3-538b-4812-9982-9c068f93a1ae" realizes "530f8d5b-90c2-4787-ba1e-676ba7937c69"
trace "8e0d22ac-7506-4a2e-8bc2-7a97f94fa52b" realizes "98cb1109-6bb4-4592-bf25-99bdc93db83a"
trace "0b3cab56-9c1e-418b-a5ed-049179978bb9" realizes "2cad898f-4c0b-4956-a24f-b86e257272c0"
trace "010959eb-bbfe-40fa-a2fa-fa2683769b48" realizes "cc53c968-ebe2-4a64-a257-e1647ce7b1e2"
trace "06c03c7d-5648-44b2-a6b9-336377d27d8d" realizes "ac9e88f4-8781-42b0-bb44-c8d725c39b92"
trace "46ffbaa1-f974-4536-bae1-3d0de536ec18" realizes "cbc21308-951f-4d40-b2e8-50d8816a1d5d"
trace "dbe313f4-8d74-44d6-8384-bf3d1d08f0c7" realizes "c2d17c2a-ff11-417e-989f-48b54d4388a4"
trace "3b738b12-a93b-4d8c-b015-9a8da58a8494" realizes "31bc5010-decb-4d62-96eb-1628496778de"
trace "4d55bc07-f92d-4fca-9c67-faaa929b45d2" realizes "eb002c8e-031e-4129-8a22-a4f175353b9e"
trace "9c35a329-9aae-47e9-8536-b4edbac5d3e5" realizes "8212d143-bc97-4152-8351-646edc65a4c9"
trace "ac16ed40-d6fd-4df7-84f9-202ec5c8af13" realizes "fc741435-84fa-457c-b116-13d8c0f15249"
trace "2903db78-bb66-4227-b884-69222e103795" realizes "7b1d7bc7-ec98-4f52-a99c-418a36980d63"
trace "42b917a3-2f0a-46a7-acbe-d7a1c4c4354d" realizes "544601f7-fbc3-4077-918c-b9199e7452e1"
trace "f8152993-9e77-4021-a05f-71b4c6a5c843" realizes "9a9a0ba6-1725-4026-92da-dcb18e1b0f4e"
trace "19ae7d7c-e19b-402e-a346-c4fec926cc2e" realizes "83a3c9ae-1cb3-4cab-885f-de529d4585ea"
trace "18fcdf21-f3b3-45ac-b0cb-966394a90b7f" realizes "b00d3b7f-56ac-4491-921e-fe1785c1c228"
trace "d5ba5cfa-5020-4b2c-885d-e33eb3afebb4" realizes "a1b30786-768a-4789-a8c7-030dfd544d69"
trace "bb5c3c57-edd1-43d8-86d0-9e46e72ce11d" realizes "654aacca-49b2-49fc-949c-ed6a871c6e42"
trace "e207d8b6-c76d-494d-9519-b7f0cac18956" realizes "4d2e526a-2166-4430-96e4-93e87b447b02"
trace "a91fa283-42a3-4415-9274-9d09244ab3e6" realizes "77bbd9a8-d804-4b57-aab6-77bd6c5add42"
trace "37794c77-aa6a-46ee-acbb-f75b65f491aa" realizes "becb1a69-c67a-40b4-8d9e-2ff2c0f6761e"
trace "e2e69198-7a61-4ad1-bf57-2d01856924c1" realizes "3b5f13a1-3d95-4fa5-95e8-4724c2e4a063"
trace "3cbdd9e7-076b-49e2-a439-ac48669d54fe" realizes "6f60db60-8fc5-427f-9b15-fc4a6373d642"
trace "10adb5f8-049d-4ea2-b5fa-87a1e23a6b6b" realizes "589697fe-d350-452c-9afa-5cdd98027e5e"
trace "ca58ed40-1dcf-4875-84e0-d4548351eea1" realizes "68708a1d-448b-483f-96cd-4478cc3ff088"
trace "cf35086b-7e31-4002-ad31-abca2fe14922" realizes "02a756c6-5f3e-41aa-80ee-b6f7049b959f"
trace "210a7607-bb1a-4a05-9e47-dbc8e977ffe0" realizes "6265c622-964e-4caa-bf7d-49a2d4b48ad0"
trace "1c787f88-e2a9-44a1-94ac-7dde8e02d64e" realizes "07d33b69-6881-40bd-9573-0f2d3e790544"
trace "098c7a52-a4f7-4b9a-b0ec-9c3b02b7687e" realizes "f4478adf-d442-4e3b-a3bc-bc108f92957e"
trace "24ebc544-379b-4899-a652-6e4ad07835c4" realizes "bd658ea3-c425-4844-9ba1-31131e38d0dc"
trace "b3f119eb-f8a5-47c4-8737-bd2ad6f81b84" realizes "9d22d9ef-e662-40d2-afab-1ee76f49b4af"
trace "bfa5d01d-8470-4c88-876f-a35f2f749e1f" realizes "55b24053-decd-4e39-9fe6-dced898b454f"
trace "a9c000e7-3c6d-45e4-a69c-c774cd5f9af5" realizes "a2bbd112-1fc6-40aa-baa7-6e8e5f645084"
trace "b9f1751a-2fe1-4abc-b02c-775b6971ddd4" realizes "70738147-7b2b-4aa0-a87b-f4942fd00998"
trace "fffd52c1-8a54-4c09-8929-ce60715f52cc" realizes "f1f7e78f-7745-4687-a89b-229520c41a8d"
trace "59d829d5-b11e-4680-be41-a84d249fe470" realizes "89c75525-66d8-4da4-a704-9641017c30d1"
trace "b43248e4-c661-45c7-87d7-ec367925b85e" realizes "b78c85ec-d3c9-43f0-a006-aa5340599509"
trace "1dd52651-c4ce-4253-94ee-2ab5daa2937d" realizes "42c74c0c-9903-4d3c-9372-bdbaf742257b"
trace "e13c6b16-a404-43be-9abf-5a7fb7d15dc7" realizes "a565c017-0a94-4788-baf4-835bf3d03205"
trace "2fa3bc43-7d00-4b51-ad50-38854356e0d0" realizes "0e1ff1e8-00c9-4cb9-8d96-7cdc2ebcf735"
trace "f5307d6d-b27f-4f2d-8817-13ee254f0011" realizes "8c966c78-51fb-4436-b36e-cc857783924d"
trace "20bd2597-5ddc-4c0d-8f77-2acdf1e96712" realizes "c385bac1-ff71-4f7c-b203-40e7779a63f8"
trace "37feeb44-e244-49db-b064-9e2e33f44f3d" realizes "7b3d0795-e518-4b28-9804-cbb023e2640b"
trace "4a0655f4-d7ed-407c-80e1-e5fc564d893c" realizes "86d0f636-e27f-436d-9e58-0ce047bd6cdd"
trace "249a8a73-3da3-46c3-92e4-b88f0114dcf3" realizes "0ad613f5-c36e-4cb8-b9f9-ce38b8078929"
trace "e4f320ab-7513-45d9-a259-2f035067664c" realizes "6e91454a-ca43-4d1e-b8f8-22435ec9c59a"
trace "4610b9a6-ecd4-4f0d-880a-4d3a81793f1c" realizes "1f6c53f8-243d-4981-b1f7-37c597cde3d3"
trace "8cce0d07-4c29-4816-90db-8e4cf51287ad" realizes "91642f3d-3687-43e4-946c-71541aa590c5"
trace "21fb7730-03d0-44c7-ac6d-8f532c57a3da" realizes "7677133e-57ee-4827-8cbf-73ee1f55a5a4"
trace "fb38d44c-4891-4a0a-b1ed-f7215109c02f" realizes "cdc4500f-0b03-4016-97eb-6094a12202aa"
trace "00c1bc21-a090-473b-8a41-af5987bfe9b5" realizes "6d34ef5b-feba-4360-8812-382fe34c0bd3"
trace "ee003445-2cc3-47b3-a7b1-2891d7eb8ca1" realizes "b80abaa1-09fb-4e6f-9033-84a8e7dd0dd7"
trace "27dd06a2-8e72-4694-9fbe-f168f0bc8198" realizes "66ca912b-4dc1-4616-9a24-c24fef5ac11e"
trace "e3a13e8a-9c88-4a6a-82bc-03f0c2ac8bbd" realizes "d8ae3120-7282-4314-bf35-41dd4c2e82f6"
trace "fa61847f-3abe-4ab2-a857-72bd5e6b4d01" realizes "8ab8e6a5-2c1f-4046-8aab-b44dd550b833"
trace "fabe2bde-f0f1-4275-a0b0-82bf22e55819" realizes "929f4bd6-ff1c-43d4-8361-abd7382f5d91"
trace "7c8c6379-58d3-45af-b57f-e571c28aade9" realizes "256a5004-f2cc-4c22-a4d4-715d4a775764"
trace "70bdc1ce-928c-4117-98c0-2094a46701bb" realizes "6bce3c7b-9f42-439d-adcf-5e47055d355a"
trace "42c63e90-bab0-45ba-b28f-e7384174f179" realizes "8026ed14-01f5-42c4-8c69-d36afeee1075"
trace "4d63300c-14d0-4d28-9cd1-af3a6666088a" realizes "f0fba710-c33e-43c7-8a55-a4f7cb82a17a"
trace "773ef288-a678-43c7-93a1-e48c0aa06593" realizes "0e7dae41-7386-402e-9e33-213a1bae69a2"
trace "0a2779ea-7dd6-4f2c-96d9-c9b18788dbd4" realizes "bfd7c07a-c578-4064-841b-85663f2096ba"
trace "d6c15a62-01ff-42ca-ad7b-65a726119365" realizes "83421190-a4d9-4ae5-8281-3b8ca9ebeada"
trace "9c056aec-462c-4a65-b7a2-0de60056fae6" realizes "9431762e-b19e-4c66-aa71-3b1e681f3472"
trace "8f4ee259-aa87-4f0e-b867-56fedf4bac7e" realizes "2c650bc2-d254-4026-b5cf-c2a001f289bd"
trace "cd8c1d42-d6a0-413b-b741-ff42f3d6a375" realizes "9615df54-b339-4ff1-bd73-b16fb15ae35f"
trace "fc974a03-0fc7-48df-aae6-80156f9920a8" realizes "56f511e4-5d06-42d3-b96e-d5a2522f6bed"
trace "0a4c9c3f-cd72-4d96-b87b-e2be6e3a9b5f" realizes "8fe69f90-7472-4e05-ab57-1e0fa4e511a4"
trace "c4a51bed-c9eb-4784-8b30-f3fa4e19a51c" realizes "849047d6-233a-4f85-bf60-8df1f145cda9"
trace "a4edf6d8-4143-499c-9e56-57c95d970304" realizes "3def4748-6a03-4e18-8004-0cacb9f3b457"
trace "50f542b7-d31f-44cc-8cf6-9a153d604ea7" realizes "5eb59259-7021-4c20-805c-9115a89867a0"
trace "6b2127cf-5024-4ab1-a1ce-cd743982ccde" realizes "24ca4fcd-9fe9-404c-8ce8-57dd25b038dc"
trace "f53e78f2-08f5-437f-964b-583a12ecf181" realizes "9601e912-9538-4dde-9c4e-c631179e763c"
trace "080f330d-2364-4093-8f09-ffecfe2fffd0" realizes "24ca4fcd-9fe9-404c-8ce8-57dd25b038dc"
trace "00dc870a-b249-493c-8646-4bb995e3a529" realizes "eecdad2c-ece1-44cb-86e1-776d8d80b61f"
trace "fe817497-e397-4053-bda2-4e4684bdcbbd" realizes "1656f21f-602e-4a9b-8f53-926f95ff219c"
trace "597598e6-3eaa-4a66-bd87-209a8509b056" realizes "6c60fb24-82ff-482a-87db-58f00e0323f4"
trace "39eb29ec-2085-47fe-b3ea-55027fc82faf" realizes "6171dcbd-85b7-4452-a035-8ab9c1be37e7"
trace "c7a28614-4584-40b8-90b4-f452d5f458b2" realizes "751ab990-c2ad-4a61-aa75-ef0ae29f9a6f"
trace "457ecf32-9740-41ee-95e7-dc65266d2b8c" realizes "b52caea4-8fdb-4bb1-b3c1-0ab7868c2ffe"
trace "f59d626b-a356-47dc-8ced-b1803f86427d" realizes "890dbd43-d1f3-47fc-98f4-a53f3e2f3c93"
trace "4c9a16e8-ed00-49cd-bab9-177a83db8d86" realizes "42b447a2-5846-40f8-ad76-2ab9807a85ed"
trace "09d7f38a-61ba-4d20-b7f1-a7b6cf3b5353" realizes "0d4ae35a-6c7c-4149-abc7-e3c08efb02b9"
trace "60761702-8e10-49a4-bb69-afaed788fcd3" realizes "42dcb187-c778-4692-b3aa-b771597b8d3d"
trace "317aed92-040e-4443-830d-7634d0f83cd0" realizes "5d72f0a9-aab2-48d3-ac82-3ab9cd693594"
trace "104510ea-dbf0-408d-b9cf-d1fba00c31cb" realizes "dd08f63d-9af0-48f2-85f5-e79141b43f11"
trace "b69efda8-a766-444d-8fab-ecd63db15685" realizes "48274f30-48fc-46dc-b4aa-2587972da645"
trace "6d479123-ab40-4f72-a21d-b28be1c0ad54" realizes "75503038-7ad6-4212-9098-b5764b7a430b"
trace "a07103d1-ed95-45f7-be53-f18fb50dbb1e" realizes "b2c226dc-43c6-494e-b6cb-dbc652561725"
trace "e18349e4-0fba-4594-b0bc-cc84bbe957fe" realizes "16d80c23-10d1-4a84-af5b-9ed35ab17471"
trace "3a658044-9895-49d6-a964-a718147cf1dd" realizes "b2f4b8ad-fe74-4318-a088-e7b00f29ed94"
trace "d7fd27a7-1d74-46b1-a702-b5f10faefe33" realizes "2d16e66e-b5e2-4880-ab96-76da319a2103"
trace "ef3fb471-ce6d-4ffc-9175-ad1d2c84cdb6" realizes "b31765b6-33cc-4df2-b6f0-63fc8fbf5b67"
trace "724a5f01-7425-490d-ae89-37a326651f30" realizes "487fa6e9-d0dc-422c-8598-4354f290b454"
trace "68b7208a-65b0-4a00-939c-89a1f3de3c04" realizes "ae0fde45-df0e-4731-b2a0-de26f1d5cb6a"
trace "aa1e5712-9787-4a92-bfd2-b0e0866ddd3d" realizes "ff789a3b-5368-468d-9bae-6f33e1fc77cb"
trace "e0da2ce7-c4ef-49de-921c-100d0b130da9" realizes "329d833a-cc85-4d3e-8dad-4a4d14192c57"
trace "a16ecc63-804b-47ba-88d3-878a32f0b4dc" realizes "10799b35-8076-42f4-8ebc-aca4c16492b5"
trace "d8678f49-b2f5-446f-bb8e-48c3c735f436" realizes "533c85d1-0b0f-4c76-8c31-041a87179d12"
trace "23b0b3c6-69e9-46ef-ab7c-91ebc1ea2eb9" realizes "57d28e0e-0696-4a01-b372-c63acc06f75f"
trace "10b05d4a-0c62-400a-9e75-25592b7f5ff3" realizes "feff77c7-6c73-4525-8f90-7943b89a1d8a"
trace "19fa1b59-c961-4897-9c20-87e61b7e91c8" realizes "689ed0dd-8f30-41a5-b228-0ca8895ccef3"
trace "a6858931-fedd-4121-a5d6-c14742de4251" realizes "83510d8c-3de1-42c3-8bbf-20f0d5cfcdde"
trace "5cddbe8c-c99e-4f3d-8f65-05de353dfa55" realizes "4243ffb3-5f36-4ee3-81bc-d786f8b9424e"
trace "1111bdb8-3546-4266-8d7e-7311c26d2364" realizes "0b71ac24-f73d-4aac-a730-3d6c57ca62bc"
trace "c8f57673-e7db-49e2-9850-2a3433e95a89" realizes "2946a0e8-ef0c-4da0-9760-f202d4298f4d"
trace "ee4a731b-d142-41f2-9a8f-679c5426b7fe" realizes "ed01d361-f9cb-4c26-a0aa-e9338d4066ff"
trace "bfed98b8-17a1-411b-b5d2-06a282a1a1ce" realizes "49555c2f-b9c9-4ed5-b704-aab0ad79080f"
trace "6107e0ce-8242-4526-b609-b54247d998be" realizes "3c6b34c9-a875-4cc6-9cb6-02b1b4b94432"
trace "afc7e130-d4ac-4edd-acb2-6ef2fa45c556" realizes "13b3665b-619f-4678-8a6f-ccef076b3552"
trace "0a65b216-6ce5-4d16-b4e1-1cfa89dbbe67" realizes "ff567a76-0861-45b6-8d64-293758b0dd7f"
trace "acdf1a44-f0cd-4a0a-ad11-f5f1f35d6756" realizes "26009e33-1109-4c48-8df9-44c5a9b6d26f"
trace "e89bd0b4-dd20-45ae-aefa-dee8f37a5ed9" realizes "88ef9fad-5981-429d-97c4-ceb49a174c17"
trace "f59d395d-fd14-479f-81ca-3546acf8dc90" realizes "19998744-6825-4911-be1d-e1c973a04511"
trace "3db2a933-36c3-4a49-a597-ae58972ffe83" realizes "39e0c808-34e0-4c9e-b82a-85bd2d11b57c"
trace "80925aa4-9aaa-4387-9670-c62f4130fc70" realizes "07c746af-a76b-4466-820b-3773214be640"
trace "159da3e6-0b3a-412d-a659-778575dbdde0" realizes "e176b21f-d560-4bb2-8fda-f0ec8f9bdb87"
trace "a8797b2f-b6f7-4bc2-9613-5fc341391c23" realizes "e1589350-78e9-43b7-b87d-926def3fdddf"
trace "6cfd0627-b1d6-45db-8fcf-9998fe24fdcc" realizes "07ab7e94-cc45-4ad3-8a48-ed3f7fed0ec9"
trace "e5f3db9a-a89a-4323-a8c7-c92dcf099d71" realizes "7104c85d-4fb0-48c6-a16b-2c162fd57a0f"
trace "e6a51ff2-0533-448c-89ea-ff3ea4aea367" realizes "7b4f505b-4cf7-4a06-9fba-cda38f4d515a"
trace "6a5cba3a-1af0-4222-be1f-5fc38999d490" realizes "7d63a0c2-2013-4457-9043-6151dceb7afe"
trace "3834215b-f255-4c38-bcc9-1310a4c0f5fd" realizes "d6f870b8-c0f4-4c90-994f-0fcda94b2969"
trace "e987d235-53f5-44c5-96ec-084c9d807ff1" realizes "6c7bfe2c-addc-473d-9381-443727c3f1c2"
trace "775785c2-9a6b-440b-8074-b8e66dd6be07" realizes "effbd5e4-a19c-4f63-88fa-3c03b0932efa"
trace "c173ec6c-22c9-4539-9ed5-1b16d0479abb" realizes "2ef387c7-2147-4275-9fe3-d6a3faea5be5"
trace "e334b6e2-ced6-40a3-8520-1531b1b0e5c5" realizes "4596aea0-9ef1-4f03-bf26-d9d61fb259e3"
trace "a11bc62b-d94b-47ea-88c2-96bedc968cff" realizes "76b010f4-8d70-4b39-89ca-cbd2c842e967"
trace "1d221aeb-40b6-4327-9a44-0392e58bebaf" realizes "1c19355c-6b5b-495c-bc09-ff91582a8d56"
trace "Provide Audio and Video Intercommunication Means / [ES] Perform Audio Announcement" realizes "Provide Audio and Video Intercommunication Means / [FS] Perform Audio Announcement"
trace "Provide Audio and Video Intercommunication Means / [FS] Perform Audio Announcement (2)" realizes "Provide Audio and Video Intercommunication Means / [FS] Perform Audio Announcement"
trace "Provide Audio and Video Intercommunication Means / [ES] Perform Audio Announcement (2)" realizes "Provide Audio and Video Intercommunication Means / [FS] Perform Audio Announcement (2)"
trace "Provide Video Entertainment Services / [ES] Start Playing VOD Movie (2)" realizes "Provide Video Entertainment Services / [ES] Start Playing VOD Movie"
trace "[ES] Select VOD Movie" realizes "Provide Video Entertainment Services / [ES] Start Playing VOD Movie"
trace "Integrate Aircraft Constraints / [FS] Handle Cabin Decompression Event (2)" realizes "Integrate Aircraft Constraints / [FS] Handle Cabin Decompression Event"
trace "Provide Access Management Control / [FS] Determine Passenger Service Availability (2)" realizes "Provide Access Management Control / [FS] Determine Passenger Service Availability"
trace "254072d9-3ac4-4dc6-8261-21e5ecc12710" -> "0ad613f5-c36e-4cb8-b9f9-ce38b8078929" { trace_type: "allocates" }
trace "254072d9-3ac4-4dc6-8261-21e5ecc12710" -> "9615df54-b339-4ff1-bd73-b16fb15ae35f" { trace_type: "allocates" }
trace "8ae83bf8-69e8-4318-8771-fb8627b576f2" -> "929f4bd6-ff1c-43d4-8361-abd7382f5d91" { trace_type: "allocates" }
trace "8ae83bf8-69e8-4318-8771-fb8627b576f2" -> "8ab8e6a5-2c1f-4046-8aab-b44dd550b833" { trace_type: "allocates" }
trace "d3ac3168-a54a-40e0-b517-9a154d41dc54" -> "a2bbd112-1fc6-40aa-baa7-6e8e5f645084" { trace_type: "allocates" }
trace "d3ac3168-a54a-40e0-b517-9a154d41dc54" -> "70738147-7b2b-4aa0-a87b-f4942fd00998" { trace_type: "allocates" }
trace "d3ac3168-a54a-40e0-b517-9a154d41dc54" -> "f1f7e78f-7745-4687-a89b-229520c41a8d" { trace_type: "allocates" }
trace "fa8dcb53-6ff5-4237-868b-2396a9fbc47c" -> "472103e7-6717-4c09-96dc-a9b4f4b3f221" { trace_type: "allocates" }
trace "fa8dcb53-6ff5-4237-868b-2396a9fbc47c" -> "99801e2d-2ff3-4e61-bd8b-5fb26601bf96" { trace_type: "allocates" }
trace "1aa5b87b-7c2e-47d8-a88b-9f14c6ef5a92" -> "9a9a0ba6-1725-4026-92da-dcb18e1b0f4e" { trace_type: "allocates" }
trace "f834b55d-a014-4f13-8bbb-3b7113a87439" -> "2eb58ffb-766e-450d-b436-5d1228203493" { trace_type: "allocates" }
trace "f834b55d-a014-4f13-8bbb-3b7113a87439" -> "845007fe-fe45-46ec-a109-47742ac52529" { trace_type: "allocates" }
trace "bd61946c-3edf-4f99-a345-061febcafc3a" -> "55b24053-decd-4e39-9fe6-dced898b454f" { trace_type: "allocates" }
trace "bd61946c-3edf-4f99-a345-061febcafc3a" -> "f0fba710-c33e-43c7-8a55-a4f7cb82a17a" { trace_type: "allocates" }
trace "bd61946c-3edf-4f99-a345-061febcafc3a" -> "9d22d9ef-e662-40d2-afab-1ee76f49b4af" { trace_type: "allocates" }
trace "a3ddc9ab-45fe-4157-8db4-f355ad100eac" -> "127898d1-536b-4fbe-a630-5f0109d15cea" { trace_type: "allocates" }
trace "a3ddc9ab-45fe-4157-8db4-f355ad100eac" -> "f0ae5ac4-8841-4866-b3db-7377f0891da6" { trace_type: "allocates" }
trace "a3ddc9ab-45fe-4157-8db4-f355ad100eac" -> "6fd71d7f-b493-4678-87d1-515eccf54213" { trace_type: "allocates" }
trace "5af8d2ef-ad0f-4190-b71f-9096d9f7e5ea" -> "7b1d7bc7-ec98-4f52-a99c-418a36980d63" { trace_type: "allocates" }
trace "ce4e7c76-a2fc-46aa-a7f4-1422ec8bdb1b" -> "31bc5010-decb-4d62-96eb-1628496778de" { trace_type: "allocates" }
trace "ce4e7c76-a2fc-46aa-a7f4-1422ec8bdb1b" -> "eb002c8e-031e-4129-8a22-a4f175353b9e" { trace_type: "allocates" }
trace "ce4e7c76-a2fc-46aa-a7f4-1422ec8bdb1b" -> "c2d17c2a-ff11-417e-989f-48b54d4388a4" { trace_type: "allocates" }
trace "f58a16a1-6663-4750-904e-0ba8249d58ea" -> "df26fc8b-2837-40a0-ac8a-96ad5c39f9ac" { trace_type: "allocates" }
trace "96228350-a898-47ba-83c1-66abdbfecb36" -> "cfbfc6da-1898-4a5d-8a93-84865d97a1b3" { trace_type: "allocates" }
trace "_gzH_AI2cEd6Q36a6kYkaHQ.CP 3" -> "ee920873-7e2d-487f-a51e-76236d6b094e.FIP 1" { trace_type: "allocates" }
trace "_gzH_AI2cEd6Q36a6kYkaHQ.CP 3" -> "30d03eaa-2615-481c-bf1f-53f177d8147c.FOP 1" { trace_type: "allocates" }
trace "_gzH_AI2cEd6Q36a6kYkaHQ.CP 4" -> "4b01ac5e-7920-491d-af35-1846163e5ad9.FIP 2" { trace_type: "allocates" }
trace "_gzH_AI2cEd6Q36a6kYkaHQ.CP 4" -> "4b01ac5e-7920-491d-af35-1846163e5ad9.FIP 1" { trace_type: "allocates" }
trace "_JNwHUJY7Ed6rH5SGn-UjSg.CP 1" -> "be9b5663-23ea-4413-abeb-3d5df84c93b8.FIP 3" { trace_type: "allocates" }
trace "_JNwHUJY7Ed6rH5SGn-UjSg.CP 1" -> "be9b5663-23ea-4413-abeb-3d5df84c93b8.FOP 2" { trace_type: "allocates" }
trace "_JNwHUJY7Ed6rH5SGn-UjSg.CP 1" -> "be9b5663-23ea-4413-abeb-3d5df84c93b8.FOP 3" { trace_type: "allocates" }
trace "_JNwHUJY7Ed6rH5SGn-UjSg.CP 2" -> "be9b5663-23ea-4413-abeb-3d5df84c93b8.FOP 4" { trace_type: "allocates" }
trace "_JNwHUJY7Ed6rH5SGn-UjSg.CP 2" -> "be9b5663-23ea-4413-abeb-3d5df84c93b8.FOP 5" { trace_type: "allocates" }
trace "_JNwHUJY7Ed6rH5SGn-UjSg.CP 2" -> "be9b5663-23ea-4413-abeb-3d5df84c93b8.FIP 2" { trace_type: "allocates" }
trace "_QHUAIJY7Ed6rH5SGn-UjSg.CP 1" -> "27d03649-545b-40b0-8407-721165539005.FOP 2" { trace_type: "allocates" }
trace "_QHUAIJY7Ed6rH5SGn-UjSg.CP 1" -> "61bf6852-3f28-425d-b8ed-4d38b2b4eebb.FOP 2" { trace_type: "allocates" }
trace "_QHUAIJY7Ed6rH5SGn-UjSg.CP 2" -> "27d03649-545b-40b0-8407-721165539005.FOP 2" { trace_type: "allocates" }
trace "_QHUAIJY7Ed6rH5SGn-UjSg.CP 2" -> "61bf6852-3f28-425d-b8ed-4d38b2b4eebb.FOP 1" { trace_type: "allocates" }
trace "_QHUAIJY7Ed6rH5SGn-UjSg.CP 3" -> "27d03649-545b-40b0-8407-721165539005.FIP 3" { trace_type: "allocates" }
trace "_QHUAIJY7Ed6rH5SGn-UjSg.CP 3" -> "27d03649-545b-40b0-8407-721165539005.FOP 2" { trace_type: "allocates" }
trace "_QHUAIJY7Ed6rH5SGn-UjSg.CP 3" -> "27d03649-545b-40b0-8407-721165539005.FOP 4" { trace_type: "allocates" }
trace "_QHUAIJY7Ed6rH5SGn-UjSg.CP 4" -> "27d03649-545b-40b0-8407-721165539005.FIP 2" { trace_type: "allocates" }
trace "df043998-7fd1-4d4d-899f-7257788d644f.CP 1" -> "39f96e89-8ead-4742-bc1f-6eb0dd33990b.FOP 4" { trace_type: "allocates" }
trace "df043998-7fd1-4d4d-899f-7257788d644f.CP 2" -> "5a58fd72-1b57-41dc-8b7b-b0b8cf577236.FOP 1" { trace_type: "allocates" }
trace "df043998-7fd1-4d4d-899f-7257788d644f.CP 2" -> "5a58fd72-1b57-41dc-8b7b-b0b8cf577236.FIP 3" { trace_type: "allocates" }
trace "df043998-7fd1-4d4d-899f-7257788d644f.CP 3" -> "39f96e89-8ead-4742-bc1f-6eb0dd33990b.FOP 3" { trace_type: "allocates" }
trace "_neToII2dEd6Q36a6kYkaHQ.CP 3" -> "651179c3-78b6-4a6f-a174-74405b81b36a.FOP 5" { trace_type: "allocates" }
trace "_neToII2dEd6Q36a6kYkaHQ.CP 3" -> "651179c3-78b6-4a6f-a174-74405b81b36a.FIP 4" { trace_type: "allocates" }
trace "_neToII2dEd6Q36a6kYkaHQ.CP 3" -> "651179c3-78b6-4a6f-a174-74405b81b36a.FIP 5" { trace_type: "allocates" }
trace "_neToII2dEd6Q36a6kYkaHQ.CP 4" -> "70f1acf3-a32d-4190-b97e-3eb361a559e7.FIP 2" { trace_type: "allocates" }
trace "_neToII2dEd6Q36a6kYkaHQ.CP 4" -> "c3b17f24-ae52-413f-ace7-6baab1287b02.FIP 4" { trace_type: "allocates" }
trace "_neToII2dEd6Q36a6kYkaHQ.CP 4" -> "70f1acf3-a32d-4190-b97e-3eb361a559e7.FOP 2" { trace_type: "allocates" }
trace "_neToII2dEd6Q36a6kYkaHQ.CP 5" -> "faf3223f-3f47-4ad6-9c58-353b34e1a30f.FOP 2" { trace_type: "allocates" }
trace "_nxYVII2dEd6Q36a6kYkaHQ.CP 1" -> "7b604e03-85be-4d93-8035-8a84ef8086db.FIP 2" { trace_type: "allocates" }
trace "_nxYVII2dEd6Q36a6kYkaHQ.CP 1" -> "7b604e03-85be-4d93-8035-8a84ef8086db.FIP 1" { trace_type: "allocates" }
trace "_nxYVII2dEd6Q36a6kYkaHQ.CP 2" -> "7b604e03-85be-4d93-8035-8a84ef8086db.FIP 3" { trace_type: "allocates" }
trace "e02be1c5-a1a2-4c11-be30-259e92ef5826.CP 1" -> "b4623d17-6e62-411d-839c-2d768664a920.FIP 2" { trace_type: "allocates" }
trace "e02be1c5-a1a2-4c11-be30-259e92ef5826.CP 2" -> "b4623d17-6e62-411d-839c-2d768664a920.FIP 3" { trace_type: "allocates" }
trace "11e922f1-192b-43a5-9060-f935c26998a5.CP 1" -> "2632b523-b577-48c5-afba-3eb4ccde3254.FIP 3" { trace_type: "allocates" }
trace "11e922f1-192b-43a5-9060-f935c26998a5.CP 1" -> "2632b523-b577-48c5-afba-3eb4ccde3254.FIP 1" { trace_type: "allocates" }
trace "_XnQJQJClEd6HJInKHsIvPg.CP 3" -> "0d727549-f01f-4cc6-ae27-5f25220ece06.FIP 2" { trace_type: "allocates" }
trace "_XnQJQJClEd6HJInKHsIvPg.CP 3" -> "0d727549-f01f-4cc6-ae27-5f25220ece06.FIP 3" { trace_type: "allocates" }
trace "_XnQJQJClEd6HJInKHsIvPg.CP 3" -> "0d727549-f01f-4cc6-ae27-5f25220ece06.FOP 2" { trace_type: "allocates" }
trace "_XnQJQJClEd6HJInKHsIvPg.CP 4" -> "7069fd31-6eb0-49fa-ae52-0b78307c2748.FOP 3" { trace_type: "allocates" }
trace "_XnQJQJClEd6HJInKHsIvPg.CP 4" -> "b55db2ae-a407-4c31-8093-75f0841094eb.FIP 4" { trace_type: "allocates" }
trace "_XnQJQJClEd6HJInKHsIvPg.CP 4" -> "b55db2ae-a407-4c31-8093-75f0841094eb.FIP 1" { trace_type: "allocates" }
trace "_XnQJQJClEd6HJInKHsIvPg.CP 5" -> "7069fd31-6eb0-49fa-ae52-0b78307c2748.FOP 4" { trace_type: "allocates" }
trace "a0ceff37-72a6-4da7-a646-89425a748235.CP 1" -> "c6ed6258-be4c-4784-a6e4-3e95f9f22dc7.FOP 5" { trace_type: "allocates" }
trace "a0ceff37-72a6-4da7-a646-89425a748235.CP 1" -> "c5e641ab-a062-4856-87ae-e675061c3eb5.FIP 1" { trace_type: "allocates" }
trace "a0ceff37-72a6-4da7-a646-89425a748235.CP 2" -> "e01b39c5-f018-4d9c-af36-c83f1b9a8a81.FOP 1" { trace_type: "allocates" }
trace "a0ceff37-72a6-4da7-a646-89425a748235.CP 2" -> "4d1c1e5c-4c96-41fc-a4a5-4bd74085f320.FOP 1" { trace_type: "allocates" }
trace "c1cbb014-c2cd-403b-a9c1-62f438e79c8c.PP 2" -> "3f745bea-11ae-4b8c-8a06-930f2391565e.CP 1" { trace_type: "allocates" }
trace "6d095cde-706f-41c2-91ab-7c5189f358c4.PP 4" -> "819adf3f-da1d-4d2b-8faf-3b1d7b99503d.CP 1" { trace_type: "allocates" }
trace "282d427b-d3c9-4eb9-973c-4d45ed593234" -> "5749f82b-2323-4e33-9fa7-3253d6fdc1ee" { trace_type: "allocates" }
trace "282d427b-d3c9-4eb9-973c-4d45ed593234" -> "d66928f0-fe85-4e75-ab12-a750b1c832cb" { trace_type: "allocates" }
trace "eee15697-a375-4a9d-ba93-7ec5b34bc63c.CP 1" -> "0378eba1-fb7b-4b4d-b39d-7a14ce4e5f4e.FIP 2" { trace_type: "allocates" }
trace "eee15697-a375-4a9d-ba93-7ec5b34bc63c.CP 2" -> "0378eba1-fb7b-4b4d-b39d-7a14ce4e5f4e.FOP 4" { trace_type: "allocates" }
trace "eee15697-a375-4a9d-ba93-7ec5b34bc63c.CP 3" -> "0378eba1-fb7b-4b4d-b39d-7a14ce4e5f4e.FIP 3" { trace_type: "allocates" }
trace "eee15697-a375-4a9d-ba93-7ec5b34bc63c.CP 4" -> "0378eba1-fb7b-4b4d-b39d-7a14ce4e5f4e.FOP 3" { trace_type: "allocates" }
trace "eee15697-a375-4a9d-ba93-7ec5b34bc63c.CP 4" -> "0378eba1-fb7b-4b4d-b39d-7a14ce4e5f4e.FOP 2" { trace_type: "allocates" }
trace "eee15697-a375-4a9d-ba93-7ec5b34bc63c.FIP 1" -> "0378eba1-fb7b-4b4d-b39d-7a14ce4e5f4e.FIP 1" { trace_type: "allocates" }
trace "eee15697-a375-4a9d-ba93-7ec5b34bc63c.FOP 1" -> "0378eba1-fb7b-4b4d-b39d-7a14ce4e5f4e.FOP 1" { trace_type: "allocates" }
trace "3f745bea-11ae-4b8c-8a06-930f2391565e.CP 1" -> "2dfb0a52-7568-4903-9c78-1e462848f868.FOP 1" { trace_type: "allocates" }
trace "3f745bea-11ae-4b8c-8a06-930f2391565e.CP 1" -> "98960be5-3e8f-4743-ab70-805645472512.FOP 2" { trace_type: "allocates" }
trace "3f745bea-11ae-4b8c-8a06-930f2391565e.FOP 5" -> "d53e6dd4-3926-4b25-80d1-14553a472f9e.FOP 5" { trace_type: "allocates" }
trace "3f745bea-11ae-4b8c-8a06-930f2391565e.FIP 1" -> "d53e6dd4-3926-4b25-80d1-14553a472f9e.FIP 1" { trace_type: "allocates" }
trace "3f745bea-11ae-4b8c-8a06-930f2391565e.FIP 2" -> "d53e6dd4-3926-4b25-80d1-14553a472f9e.FIP 2" { trace_type: "allocates" }
trace "647781a2-de41-4fa1-a6e6-3a886a706c4e.FOP 2" -> "ac5be58c-aa92-4247-8c8f-9867ddbd6289.FOP 2" { trace_type: "allocates" }
trace "9eef8f0f-1bf8-4e51-a17f-480b3afbfd85.CP 2" -> "c7edaab3-11bc-4409-b300-13c4b4192742.FOP 2" { trace_type: "allocates" }
trace "9eef8f0f-1bf8-4e51-a17f-480b3afbfd85.CP 2" -> "bd383ff2-258a-41dd-973f-07f7aa30da7a.FOP 1" { trace_type: "allocates" }
trace "9eef8f0f-1bf8-4e51-a17f-480b3afbfd85.CP 2" -> "c7edaab3-11bc-4409-b300-13c4b4192742.FOP 1" { trace_type: "allocates" }
trace "9eef8f0f-1bf8-4e51-a17f-480b3afbfd85.CP 2" -> "83b4863a-7795-4911-aeef-2938443cce3d.FOP1" { trace_type: "allocates" }
trace "9eef8f0f-1bf8-4e51-a17f-480b3afbfd85.CP 2" -> "c7edaab3-11bc-4409-b300-13c4b4192742.FOP 3" { trace_type: "allocates" }
trace "819adf3f-da1d-4d2b-8faf-3b1d7b99503d.CP 1" -> "4c361e00-d6d0-4acf-a523-1de11acf7797.FIP 1" { trace_type: "allocates" }
trace "819adf3f-da1d-4d2b-8faf-3b1d7b99503d.CP 1" -> "1289a361-a3e8-416b-a9e0-e4fcd528d9e3.FIP 2" { trace_type: "allocates" }
trace "abc69d28-d5c4-470e-afd1-89daa1cc81d2.CP 1" -> "cebc5976-81d6-4718-b1f0-9f8b7687ceeb.FOP2" { trace_type: "allocates" }
trace "abc69d28-d5c4-470e-afd1-89daa1cc81d2.CP 1" -> "cebc5976-81d6-4718-b1f0-9f8b7687ceeb.FOP1" { trace_type: "allocates" }

diagram "[CDB] In-Flight Entertainment Dictionary" {
    view: cdb
    shows: ["0c17402c-e5e4-44d4-9692-efc4bfb8ab2a", "69bc8e28-37ec-4b2b-b47d-0cfb9ea8e645", "8a3c3a57-1ff3-4d9e-845e-0e142c1ca7dc", "67bd0af1-6242-4dec-aeef-3f54a7d8ba47", "bcc88a58-3c68-4f46-8502-f5783a6f9046", "4bb53659-1f4f-4fc0-9eb6-8e6d7bec1415", "e3b6bcb8-d453-489f-afb1-3c7811acad55", "6bca4516-9dc6-45ad-9302-d7924289aa23"]
}

diagram "[CDB] Play Video Movie - Logical" {
    view: cdb
    shows: ["e3830e92-47f9-4a57-b9ac-708bc64f6937", "1885aa4c-b56f-48cb-a433-a135f0bd7b01", "1fdba7f2-be01-4882-b999-b65528c3ee35", "f1309e1f-e30b-47ed-8027-41f12017ef50", "abe9bb46-3a60-4f88-a4ad-42f99f1238de", "a86aeed3-084c-4618-b23a-bd385f92ecb0", "b417cf33-5d66-4249-9949-f547f7876a19", "2a07f10a-8c7b-4065-8d35-bf2b540c8348", "fbed3075-3536-4603-9089-f9571423c7cd", "dd056c2f-d8b0-4f9e-b4ba-382926d2aed4", "6c858859-acc3-4f31-a6fc-0aee4f8c07d8", "44f6d074-c280-4d8d-83e0-ced6ba8d5cf9", "a990b5ef-faa7-453d-b1f0-d46e8a15c4fd", "0ba002c1-c0d9-4e1e-98cf-60e55d02ecfd", "e5bebd43-6881-4c89-be4a-7cc97632df51", "6a78f28a-a27b-4f42-a002-04924ee4c75f", "a7059561-22f7-4589-aed8-05b33d8600c8", "6bca4516-9dc6-45ad-9302-d7924289aa23"]
}

diagram "[CDB] Play Video Movie - Physical" {
    view: cdb
    shows: ["1cd707ba-fe65-4e3c-9fff-21966d4e5716", "d3b096f5-9eb4-4c4a-b0af-946a71992c54", "a34141d6-27d6-4e31-9994-828ee1560ded", "b1ca4bf5-1b54-4f5c-a385-9514227695d3", "32703aff-a875-41e8-a93c-20e8a4b8df4b", "03c86c27-a965-4b3a-b641-e1969d4c7261", "b65b0dee-34a9-4397-9626-e8095e348482", "cb931585-e5a5-48ba-8ac8-cf3b13fffe33", "911f07f3-4c41-4351-aff4-1f1a84d9417a", "a4ecc90d-72fb-486d-8de2-90d5473df459", "d301f697-5975-41e2-97af-2aea81db5f6f", "e5d905f2-f0a3-48b6-9658-cecb923e6fb3", "c9e7c37a-e940-434c-9dab-1f1029f2cf14", "eb58eec3-b4ee-4a51-80c2-666fae882c3f", "940ce3fc-ef6f-4db0-85c2-d9866a9472a7", "fbfc173b-0081-4501-9d92-f026ff210ef2", "93f522c4-99eb-40b2-9baa-a576b6a3af30", "8b064948-3736-4410-ab07-4d09ca2c1d24", "10678425-5baa-47de-8d6b-e7ec0a47666f", "6105a469-c022-4fd3-86aa-892d8c0b5967", "e5bebd43-6881-4c89-be4a-7cc97632df51", "e02cf446-19e2-4b24-b72d-340fd3c39355", "2e048fef-989b-4332-ae3c-e47c63bbec2b", "30f1f33a-1c0d-4c1f-8ada-62f22d676917", "36986419-6217-426f-bcab-e9ad33de0a80", "89d4b45e-af13-4733-aa66-44790e984572", "bad2a514-b52f-4c98-9be1-661a1f2177e2", "7fef079a-4ee1-44ea-9388-ecaefb276599", "5865788f-9133-42d8-8cfc-8b7b31388859", "7fc204dc-9c56-4b00-a927-82a23b8bb5ca"]
}

diagram "[EAB] Configuration Items and Realized Artefacts" {
    view: pbs
    shows: ["d90f7495-1f1a-4882-95a1-edda9f16fbdf", "a5d70e56-04c7-4b44-81e4-ce891cae0da3", "64961d5d-932d-4c69-8be5-2879940483bc", "c241a991-774a-4a82-854a-91d22e804b11", "6c0ada8f-a517-4a12-bd8e-12c28ab8b378", "5e139f27-c1ea-42d5-a7f0-1cdc6d8bc52c", "f6b83d87-9b95-4a3c-aeea-48d755a6eda5", "6e738dc3-2775-4fd8-bec7-376057b88a11", "c02105c2-ca0e-44ea-8868-4bf81d6d8e1c", "79d96a1d-b45a-4621-9c30-3d793fd40ee8"]
}

diagram "[LCBD] Architecture Drivers" {
    view: lab
    shows: ["_8i1cwI2bEd6Q36a6kYkaHQ", "_8rBvcI2bEd6Q36a6kYkaHQ", "11e922f1-192b-43a5-9060-f935c26998a5", "8dbc153d-c9e8-4cb5-93af-ecf9a0fc939c", "_gzH_AI2cEd6Q36a6kYkaHQ", "_JNwHUJY7Ed6rH5SGn-UjSg", "_QHUAIJY7Ed6rH5SGn-UjSg", "df043998-7fd1-4d4d-899f-7257788d644f", "_neToII2dEd6Q36a6kYkaHQ", "_nxYVII2dEd6Q36a6kYkaHQ", "e02be1c5-a1a2-4c11-be30-259e92ef5826", "_XnQJQJClEd6HJInKHsIvPg", "_SF1a8JY-Ed6rH5SGn-UjSg", "_Rml6MJY-Ed6rH5SGn-UjSg", "_2SUXEJZFEd6rH5SGn-UjSg", "281c331f-1184-4c7d-afb2-7a55ff09a80f", "97e2a56c-0d4c-4a56-924a-63a31e6c50d7", "01a67cde-a09c-446e-9637-ab235664d23c"]
}

diagram "[LFBD] All Functions" {
    view: lab
    shows: ["bca11666-f58a-41e4-9a16-5964771b55b0", "37c0f280-8012-4736-82df-4e9114fc61d4", "82445ac7-1b27-4c0c-bdff-7fda4e1e7496", "cf623512-1614-4548-9b4c-4c0e614c4516", "aeafffae-c0e8-45a3-9ad1-105825574ced", "295db719-853a-4cb6-9bbe-2e66b5ae9ace", "63136eb0-622b-43cc-b4da-55b86ae281c1", "bcb08500-a828-43b0-ac8b-ff4c7cb3b6b4", "18da001e-dd8b-4041-ba4c-eb2211c624c4", "cd4b1bad-e12e-45eb-96fd-dfec5b483457", "55e3b1b8-dea2-4fbf-b78f-0f270009b56a", "ebce47df-f06c-4eea-b367-b828df4b44fa", "4d1c1e5c-4c96-41fc-a4a5-4bd74085f320", "9ae12cc6-8bb4-4f99-ad87-8a30a22b919b", "c6ed6258-be4c-4784-a6e4-3e95f9f22dc7", "c5e641ab-a062-4856-87ae-e675061c3eb5", "e01b39c5-f018-4d9c-af36-c83f1b9a8a81", "bf6388c4-3549-46a6-af08-48412581967e", "d5254d45-3946-431f-8162-a28a3f2263a7", "9307409a-5b76-4a96-876c-85468c60e376", "05aefe2e-d873-4fb0-8ad7-4dafc46c764e", "9d3d6fe7-16df-4382-9d49-492e4cd2b73f", "145e28dc-9fbe-4f01-a378-5af8cbe8e2d7", "c268e3f1-3cbc-4e62-8ae2-75975bc4481f", "4b01ac5e-7920-491d-af35-1846163e5ad9", "1e812d3d-6c35-42c9-80fa-eb64dfc1a3a8", "ee920873-7e2d-487f-a51e-76236d6b094e", "30d03eaa-2615-481c-bf1f-53f177d8147c", "2a6d57b4-97f3-4344-b645-cefcc9462708", "70d62f70-d0b6-491c-9e9e-f5ed77179ce4", "be9b5663-23ea-4413-abeb-3d5df84c93b8", "95495bbc-987c-4b17-95d0-9aa5a87f0344", "a8452b58-3135-457b-8d85-59bd23240297", "faf3223f-3f47-4ad6-9c58-353b34e1a30f", "7069fd31-6eb0-49fa-ae52-0b78307c2748", "d4ee89c8-dcf8-4fa5-b276-0afa03d409c9", "61bf6852-3f28-425d-b8ed-4d38b2b4eebb", "27d03649-545b-40b0-8407-721165539005", "cbe07842-a67d-488a-bf29-5b46d3369c15", "7b604e03-85be-4d93-8035-8a84ef8086db", "2632b523-b577-48c5-afba-3eb4ccde3254", "b55db2ae-a407-4c31-8093-75f0841094eb", "566f8900-7408-42d3-82f8-ba18bf0d923f", "4ff700ef-4f6b-4151-917b-e59064e5a225", "39f96e89-8ead-4742-bc1f-6eb0dd33990b", "355a6d2b-8c53-46f8-8c0b-38356b4d4271", "7a8c51b2-c42f-49cd-a003-752d0de0b4da", "94aebe8b-af13-4a7a-a7ee-8fd10a7fe489", "5165a225-e662-4b4a-95cd-c94fc5726524", "651179c3-78b6-4a6f-a174-74405b81b36a", "910a61f1-3da0-404f-9db5-c226af9a3c47", "3136218c-bf14-437d-99c9-6593352eead9", "81d017bc-85ca-4bcd-acc9-51423542c61e", "1aa42ce5-1c2c-4473-9840-7d20a60c377e", "0d727549-f01f-4cc6-ae27-5f25220ece06", "a90e98e8-741e-478a-9e14-af7a50ee2525", "11f3e0e8-a78d-4962-bed8-9672b1167331", "70f1acf3-a32d-4190-b97e-3eb361a559e7", "2ea2703a-6dd1-4835-aaae-16179864ac1d", "5a58fd72-1b57-41dc-8b7b-b0b8cf577236", "b4623d17-6e62-411d-839c-2d768664a920", "3e9ccc30-596e-40e6-8941-64838248dcee", "c3b17f24-ae52-413f-ace7-6baab1287b02"]
}

diagram "[LDFB] [CTX] Manage Passenger Services Lifecycle" {
    view: lab
    shows: ["11f3e0e8-a78d-4962-bed8-9672b1167331", "70f1acf3-a32d-4190-b97e-3eb361a559e7", "2ea2703a-6dd1-4835-aaae-16179864ac1d", "5a58fd72-1b57-41dc-8b7b-b0b8cf577236", "b4623d17-6e62-411d-839c-2d768664a920", "3e9ccc30-596e-40e6-8941-64838248dcee", "c3b17f24-ae52-413f-ace7-6baab1287b02", "63136eb0-622b-43cc-b4da-55b86ae281c1", "9d3d6fe7-16df-4382-9d49-492e4cd2b73f", "1e812d3d-6c35-42c9-80fa-eb64dfc1a3a8", "355a6d2b-8c53-46f8-8c0b-38356b4d4271", "4ff700ef-4f6b-4151-917b-e59064e5a225", "7069fd31-6eb0-49fa-ae52-0b78307c2748", "39f96e89-8ead-4742-bc1f-6eb0dd33990b"]
}

diagram "[LDFB] [CTX] Process Audio Video Requests" {
    view: lab
    shows: ["a8452b58-3135-457b-8d85-59bd23240297", "faf3223f-3f47-4ad6-9c58-353b34e1a30f", "7069fd31-6eb0-49fa-ae52-0b78307c2748", "d4ee89c8-dcf8-4fa5-b276-0afa03d409c9", "355a6d2b-8c53-46f8-8c0b-38356b4d4271", "11f3e0e8-a78d-4962-bed8-9672b1167331"]
}

diagram "[LDFB] [CTX] Broadcast Stored Audio Video Streams" {
    view: lab
    shows: ["d4ee89c8-dcf8-4fa5-b276-0afa03d409c9", "61bf6852-3f28-425d-b8ed-4d38b2b4eebb", "27d03649-545b-40b0-8407-721165539005", "cbe07842-a67d-488a-bf29-5b46d3369c15", "a8452b58-3135-457b-8d85-59bd23240297", "be9b5663-23ea-4413-abeb-3d5df84c93b8", "c268e3f1-3cbc-4e62-8ae2-75975bc4481f"]
}

diagram "[LDFB] [CTX] Display Video and Play Audio" {
    view: lab
    shows: ["cbe07842-a67d-488a-bf29-5b46d3369c15", "2632b523-b577-48c5-afba-3eb4ccde3254", "7b604e03-85be-4d93-8035-8a84ef8086db", "b55db2ae-a407-4c31-8093-75f0841094eb", "566f8900-7408-42d3-82f8-ba18bf0d923f", "4ff700ef-4f6b-4151-917b-e59064e5a225", "d4ee89c8-dcf8-4fa5-b276-0afa03d409c9", "aeafffae-c0e8-45a3-9ad1-105825574ced", "295db719-853a-4cb6-9bbe-2e66b5ae9ace", "18da001e-dd8b-4041-ba4c-eb2211c624c4", "37c0f280-8012-4736-82df-4e9114fc61d4", "39f96e89-8ead-4742-bc1f-6eb0dd33990b", "11f3e0e8-a78d-4962-bed8-9672b1167331"]
}

diagram "[LAB] IFE System - All Components, CEs" {
    view: lab
    shows: ["724d6db9-9175-450b-8f05-023fcffec79c", "_8i1cwI2bEd6Q36a6kYkaHQ", "_gzH_AI2cEd6Q36a6kYkaHQ", "_SF1a8JY-Ed6rH5SGn-UjSg", "_Rml6MJY-Ed6rH5SGn-UjSg", "_JNwHUJY7Ed6rH5SGn-UjSg", "_QHUAIJY7Ed6rH5SGn-UjSg", "df043998-7fd1-4d4d-899f-7257788d644f", "8dbc153d-c9e8-4cb5-93af-ecf9a0fc939c", "_XnQJQJClEd6HJInKHsIvPg", "97e2a56c-0d4c-4a56-924a-63a31e6c50d7", "01a67cde-a09c-446e-9637-ab235664d23c", "_8rBvcI2bEd6Q36a6kYkaHQ", "_nxYVII2dEd6Q36a6kYkaHQ", "_neToII2dEd6Q36a6kYkaHQ", "_2SUXEJZFEd6rH5SGn-UjSg", "281c331f-1184-4c7d-afb2-7a55ff09a80f", "e02be1c5-a1a2-4c11-be30-259e92ef5826", "11e922f1-192b-43a5-9060-f935c26998a5", "94f479d0-c672-4a9c-bfbb-1fb5e01a73ee", "a0ceff37-72a6-4da7-a646-89425a748235", "395ee0ab-cb3d-48e8-a411-09a98053292f", "d459ccb1-cad9-4756-bb0c-b151c60461e7"]
}

diagram "[LAB] [BUILD] Template" {
    view: lab
    shows: ["_8rBvcI2bEd6Q36a6kYkaHQ", "_nxYVII2dEd6Q36a6kYkaHQ", "3136218c-bf14-437d-99c9-6593352eead9", "7b604e03-85be-4d93-8035-8a84ef8086db", "_neToII2dEd6Q36a6kYkaHQ", "_2SUXEJZFEd6rH5SGn-UjSg", "651179c3-78b6-4a6f-a174-74405b81b36a", "c3b17f24-ae52-413f-ace7-6baab1287b02", "faf3223f-3f47-4ad6-9c58-353b34e1a30f", "910a61f1-3da0-404f-9db5-c226af9a3c47", "281c331f-1184-4c7d-afb2-7a55ff09a80f", "5165a225-e662-4b4a-95cd-c94fc5726524", "70f1acf3-a32d-4190-b97e-3eb361a559e7", "4ff700ef-4f6b-4151-917b-e59064e5a225", "94aebe8b-af13-4a7a-a7ee-8fd10a7fe489", "2ea2703a-6dd1-4835-aaae-16179864ac1d", "e02be1c5-a1a2-4c11-be30-259e92ef5826", "3e9ccc30-596e-40e6-8941-64838248dcee", "b4623d17-6e62-411d-839c-2d768664a920", "395ee0ab-cb3d-48e8-a411-09a98053292f", "63136eb0-622b-43cc-b4da-55b86ae281c1", "18da001e-dd8b-4041-ba4c-eb2211c624c4", "aeafffae-c0e8-45a3-9ad1-105825574ced", "295db719-853a-4cb6-9bbe-2e66b5ae9ace", "bcb08500-a828-43b0-ac8b-ff4c7cb3b6b4", "_8i1cwI2bEd6Q36a6kYkaHQ", "_QHUAIJY7Ed6rH5SGn-UjSg", "61bf6852-3f28-425d-b8ed-4d38b2b4eebb", "27d03649-545b-40b0-8407-721165539005", "_JNwHUJY7Ed6rH5SGn-UjSg", "be9b5663-23ea-4413-abeb-3d5df84c93b8", "70d62f70-d0b6-491c-9e9e-f5ed77179ce4", "df043998-7fd1-4d4d-899f-7257788d644f", "5a58fd72-1b57-41dc-8b7b-b0b8cf577236", "39f96e89-8ead-4742-bc1f-6eb0dd33990b", "_gzH_AI2cEd6Q36a6kYkaHQ", "_Rml6MJY-Ed6rH5SGn-UjSg", "1e812d3d-6c35-42c9-80fa-eb64dfc1a3a8", "_SF1a8JY-Ed6rH5SGn-UjSg", "4b01ac5e-7920-491d-af35-1846163e5ad9", "30d03eaa-2615-481c-bf1f-53f177d8147c", "ee920873-7e2d-487f-a51e-76236d6b094e", "8dbc153d-c9e8-4cb5-93af-ecf9a0fc939c", "_XnQJQJClEd6HJInKHsIvPg", "01a67cde-a09c-446e-9637-ab235664d23c", "7069fd31-6eb0-49fa-ae52-0b78307c2748", "0d727549-f01f-4cc6-ae27-5f25220ece06", "b55db2ae-a407-4c31-8093-75f0841094eb", "97e2a56c-0d4c-4a56-924a-63a31e6c50d7", "1aa42ce5-1c2c-4473-9840-7d20a60c377e", "566f8900-7408-42d3-82f8-ba18bf0d923f", "a90e98e8-741e-478a-9e14-af7a50ee2525", "11e922f1-192b-43a5-9060-f935c26998a5", "2632b523-b577-48c5-afba-3eb4ccde3254", "a0ceff37-72a6-4da7-a646-89425a748235", "e01b39c5-f018-4d9c-af36-c83f1b9a8a81", "c6ed6258-be4c-4784-a6e4-3e95f9f22dc7", "ebce47df-f06c-4eea-b367-b828df4b44fa", "55e3b1b8-dea2-4fbf-b78f-0f270009b56a", "c5e641ab-a062-4856-87ae-e675061c3eb5", "9ae12cc6-8bb4-4f99-ad87-8a30a22b919b", "4d1c1e5c-4c96-41fc-a4a5-4bd74085f320", "d459ccb1-cad9-4756-bb0c-b151c60461e7", "9d3d6fe7-16df-4382-9d49-492e4cd2b73f", "d5254d45-3946-431f-8162-a28a3f2263a7", "05aefe2e-d873-4fb0-8ad7-4dafc46c764e", "9307409a-5b76-4a96-876c-85468c60e376", "94f479d0-c672-4a9c-bfbb-1fb5e01a73ee", "82445ac7-1b27-4c0c-bdff-7fda4e1e7496", "37c0f280-8012-4736-82df-4e9114fc61d4"]
}

diagram "[LDFB] [CTX] Run Video-On-Demand Service" {
    view: lab
    shows: ["7a8c51b2-c42f-49cd-a003-752d0de0b4da", "5165a225-e662-4b4a-95cd-c94fc5726524", "94aebe8b-af13-4a7a-a7ee-8fd10a7fe489", "651179c3-78b6-4a6f-a174-74405b81b36a", "910a61f1-3da0-404f-9db5-c226af9a3c47", "3136218c-bf14-437d-99c9-6593352eead9", "be9b5663-23ea-4413-abeb-3d5df84c93b8", "95495bbc-987c-4b17-95d0-9aa5a87f0344", "11f3e0e8-a78d-4962-bed8-9672b1167331", "bcb08500-a828-43b0-ac8b-ff4c7cb3b6b4"]
}

diagram "[LDFB] Top Level Functions" {
    view: lab
    shows: ["2a6d57b4-97f3-4344-b645-cefcc9462708", "355a6d2b-8c53-46f8-8c0b-38356b4d4271", "95495bbc-987c-4b17-95d0-9aa5a87f0344", "cf623512-1614-4548-9b4c-4c0e614c4516", "bf6388c4-3549-46a6-af08-48412581967e", "bca11666-f58a-41e4-9a16-5964771b55b0", "cd4b1bad-e12e-45eb-96fd-dfec5b483457", "11f3e0e8-a78d-4962-bed8-9672b1167331", "c268e3f1-3cbc-4e62-8ae2-75975bc4481f"]
}

diagram "[LDFB] [CTX] Manage Audio and Video Diffusion" {
    view: lab
    shows: ["95495bbc-987c-4b17-95d0-9aa5a87f0344", "d4ee89c8-dcf8-4fa5-b276-0afa03d409c9", "cbe07842-a67d-488a-bf29-5b46d3369c15", "a8452b58-3135-457b-8d85-59bd23240297", "39f96e89-8ead-4742-bc1f-6eb0dd33990b", "355a6d2b-8c53-46f8-8c0b-38356b4d4271", "be9b5663-23ea-4413-abeb-3d5df84c93b8", "37c0f280-8012-4736-82df-4e9114fc61d4", "aeafffae-c0e8-45a3-9ad1-105825574ced", "295db719-853a-4cb6-9bbe-2e66b5ae9ace", "18da001e-dd8b-4041-ba4c-eb2211c624c4", "c268e3f1-3cbc-4e62-8ae2-75975bc4481f", "11f3e0e8-a78d-4962-bed8-9672b1167331", "c5e641ab-a062-4856-87ae-e675061c3eb5"]
}

diagram "[LDFB] [CTX] Run Cabin Intercommunication Service" {
    view: lab
    shows: ["81d017bc-85ca-4bcd-acc9-51423542c61e", "1aa42ce5-1c2c-4473-9840-7d20a60c377e", "a90e98e8-741e-478a-9e14-af7a50ee2525", "0d727549-f01f-4cc6-ae27-5f25220ece06", "2a6d57b4-97f3-4344-b645-cefcc9462708", "95495bbc-987c-4b17-95d0-9aa5a87f0344", "37c0f280-8012-4736-82df-4e9114fc61d4"]
}

diagram "[LAB] [BUILD] All Components, Functions, CEs, FEs" {
    view: lab
    shows: ["_8rBvcI2bEd6Q36a6kYkaHQ", "_nxYVII2dEd6Q36a6kYkaHQ", "3136218c-bf14-437d-99c9-6593352eead9", "7b604e03-85be-4d93-8035-8a84ef8086db", "_neToII2dEd6Q36a6kYkaHQ", "_2SUXEJZFEd6rH5SGn-UjSg", "651179c3-78b6-4a6f-a174-74405b81b36a", "c3b17f24-ae52-413f-ace7-6baab1287b02", "faf3223f-3f47-4ad6-9c58-353b34e1a30f", "910a61f1-3da0-404f-9db5-c226af9a3c47", "281c331f-1184-4c7d-afb2-7a55ff09a80f", "5165a225-e662-4b4a-95cd-c94fc5726524", "70f1acf3-a32d-4190-b97e-3eb361a559e7", "4ff700ef-4f6b-4151-917b-e59064e5a225", "94aebe8b-af13-4a7a-a7ee-8fd10a7fe489", "2ea2703a-6dd1-4835-aaae-16179864ac1d", "e02be1c5-a1a2-4c11-be30-259e92ef5826", "3e9ccc30-596e-40e6-8941-64838248dcee", "b4623d17-6e62-411d-839c-2d768664a920", "395ee0ab-cb3d-48e8-a411-09a98053292f", "63136eb0-622b-43cc-b4da-55b86ae281c1", "18da001e-dd8b-4041-ba4c-eb2211c624c4", "aeafffae-c0e8-45a3-9ad1-105825574ced", "295db719-853a-4cb6-9bbe-2e66b5ae9ace", "bcb08500-a828-43b0-ac8b-ff4c7cb3b6b4", "_8i1cwI2bEd6Q36a6kYkaHQ", "_QHUAIJY7Ed6rH5SGn-UjSg", "61bf6852-3f28-425d-b8ed-4d38b2b4eebb", "27d03649-545b-40b0-8407-721165539005", "_JNwHUJY7Ed6rH5SGn-UjSg", "be9b5663-23ea-4413-abeb-3d5df84c93b8", "70d62f70-d0b6-491c-9e9e-f5ed77179ce4", "df043998-7fd1-4d4d-899f-7257788d644f", "5a58fd72-1b57-41dc-8b7b-b0b8cf577236", "39f96e89-8ead-4742-bc1f-6eb0dd33990b", "_gzH_AI2cEd6Q36a6kYkaHQ", "_Rml6MJY-Ed6rH5SGn-UjSg", "1e812d3d-6c35-42c9-80fa-eb64dfc1a3a8", "_SF1a8JY-Ed6rH5SGn-UjSg", "4b01ac5e-7920-491d-af35-1846163e5ad9", "30d03eaa-2615-481c-bf1f-53f177d8147c", "ee920873-7e2d-487f-a51e-76236d6b094e", "8dbc153d-c9e8-4cb5-93af-ecf9a0fc939c", "_XnQJQJClEd6HJInKHsIvPg", "01a67cde-a09c-446e-9637-ab235664d23c", "7069fd31-6eb0-49fa-ae52-0b78307c2748", "0d727549-f01f-4cc6-ae27-5f25220ece06", "b55db2ae-a407-4c31-8093-75f0841094eb", "97e2a56c-0d4c-4a56-924a-63a31e6c50d7", "1aa42ce5-1c2c-4473-9840-7d20a60c377e", "566f8900-7408-42d3-82f8-ba18bf0d923f", "a90e98e8-741e-478a-9e14-af7a50ee2525", "11e922f1-192b-43a5-9060-f935c26998a5", "2632b523-b577-48c5-afba-3eb4ccde3254", "a0ceff37-72a6-4da7-a646-89425a748235", "e01b39c5-f018-4d9c-af36-c83f1b9a8a81", "c6ed6258-be4c-4784-a6e4-3e95f9f22dc7", "ebce47df-f06c-4eea-b367-b828df4b44fa", "55e3b1b8-dea2-4fbf-b78f-0f270009b56a", "c5e641ab-a062-4856-87ae-e675061c3eb5", "9ae12cc6-8bb4-4f99-ad87-8a30a22b919b", "4d1c1e5c-4c96-41fc-a4a5-4bd74085f320", "d459ccb1-cad9-4756-bb0c-b151c60461e7", "9d3d6fe7-16df-4382-9d49-492e4cd2b73f", "d5254d45-3946-431f-8162-a28a3f2263a7", "05aefe2e-d873-4fb0-8ad7-4dafc46c764e", "9307409a-5b76-4a96-876c-85468c60e376", "94f479d0-c672-4a9c-bfbb-1fb5e01a73ee", "82445ac7-1b27-4c0c-bdff-7fda4e1e7496", "37c0f280-8012-4736-82df-4e9114fc61d4"]
}

diagram "[LDFB] [CTX] Run Passenger Services" {
    view: lab
    shows: ["355a6d2b-8c53-46f8-8c0b-38356b4d4271", "7a8c51b2-c42f-49cd-a003-752d0de0b4da", "81d017bc-85ca-4bcd-acc9-51423542c61e", "bca11666-f58a-41e4-9a16-5964771b55b0", "cf623512-1614-4548-9b4c-4c0e614c4516", "11f3e0e8-a78d-4962-bed8-9672b1167331", "2a6d57b4-97f3-4344-b645-cefcc9462708", "95495bbc-987c-4b17-95d0-9aa5a87f0344"]
}

diagram "[LAB][CTX] Watch Imposed Video on Cabin Screen FC" {
    view: lab
    shows: ["395ee0ab-cb3d-48e8-a411-09a98053292f", "295db719-853a-4cb6-9bbe-2e66b5ae9ace", "_8i1cwI2bEd6Q36a6kYkaHQ", "_QHUAIJY7Ed6rH5SGn-UjSg", "27d03649-545b-40b0-8407-721165539005", "61bf6852-3f28-425d-b8ed-4d38b2b4eebb", "_JNwHUJY7Ed6rH5SGn-UjSg", "be9b5663-23ea-4413-abeb-3d5df84c93b8", "_gzH_AI2cEd6Q36a6kYkaHQ", "_Rml6MJY-Ed6rH5SGn-UjSg", "_SF1a8JY-Ed6rH5SGn-UjSg", "30d03eaa-2615-481c-bf1f-53f177d8147c", "8dbc153d-c9e8-4cb5-93af-ecf9a0fc939c", "_XnQJQJClEd6HJInKHsIvPg", "01a67cde-a09c-446e-9637-ab235664d23c", "7069fd31-6eb0-49fa-ae52-0b78307c2748", "b55db2ae-a407-4c31-8093-75f0841094eb", "97e2a56c-0d4c-4a56-924a-63a31e6c50d7", "1aa42ce5-1c2c-4473-9840-7d20a60c377e", "566f8900-7408-42d3-82f8-ba18bf0d923f", "11e922f1-192b-43a5-9060-f935c26998a5", "2632b523-b577-48c5-afba-3eb4ccde3254", "a0ceff37-72a6-4da7-a646-89425a748235", "c5e641ab-a062-4856-87ae-e675061c3eb5", "94f479d0-c672-4a9c-bfbb-1fb5e01a73ee", "37c0f280-8012-4736-82df-4e9114fc61d4"]
}

diagram "[LAB][CTX] Start Playing VOD Movie FC" {
    view: lab
    shows: ["_8rBvcI2bEd6Q36a6kYkaHQ", "_nxYVII2dEd6Q36a6kYkaHQ", "3136218c-bf14-437d-99c9-6593352eead9", "7b604e03-85be-4d93-8035-8a84ef8086db", "_neToII2dEd6Q36a6kYkaHQ", "_2SUXEJZFEd6rH5SGn-UjSg", "faf3223f-3f47-4ad6-9c58-353b34e1a30f", "281c331f-1184-4c7d-afb2-7a55ff09a80f", "5165a225-e662-4b4a-95cd-c94fc5726524", "395ee0ab-cb3d-48e8-a411-09a98053292f", "aeafffae-c0e8-45a3-9ad1-105825574ced", "bcb08500-a828-43b0-ac8b-ff4c7cb3b6b4", "_8i1cwI2bEd6Q36a6kYkaHQ", "_QHUAIJY7Ed6rH5SGn-UjSg", "61bf6852-3f28-425d-b8ed-4d38b2b4eebb", "27d03649-545b-40b0-8407-721165539005", "_JNwHUJY7Ed6rH5SGn-UjSg", "be9b5663-23ea-4413-abeb-3d5df84c93b8"]
}

diagram "[LDFB] [CTX] Provide Aircraft Interface" {
    view: lab
    shows: ["c268e3f1-3cbc-4e62-8ae2-75975bc4481f", "4b01ac5e-7920-491d-af35-1846163e5ad9", "1e812d3d-6c35-42c9-80fa-eb64dfc1a3a8", "ee920873-7e2d-487f-a51e-76236d6b094e", "30d03eaa-2615-481c-bf1f-53f177d8147c", "cd4b1bad-e12e-45eb-96fd-dfec5b483457", "11f3e0e8-a78d-4962-bed8-9672b1167331", "39f96e89-8ead-4742-bc1f-6eb0dd33990b", "61bf6852-3f28-425d-b8ed-4d38b2b4eebb"]
}

diagram "[LAB][CTX] Broadcast Audio Announcement FC" {
    view: lab
    shows: ["_8rBvcI2bEd6Q36a6kYkaHQ", "_nxYVII2dEd6Q36a6kYkaHQ", "3136218c-bf14-437d-99c9-6593352eead9", "7b604e03-85be-4d93-8035-8a84ef8086db", "_neToII2dEd6Q36a6kYkaHQ", "_2SUXEJZFEd6rH5SGn-UjSg", "910a61f1-3da0-404f-9db5-c226af9a3c47", "281c331f-1184-4c7d-afb2-7a55ff09a80f", "4ff700ef-4f6b-4151-917b-e59064e5a225", "e02be1c5-a1a2-4c11-be30-259e92ef5826", "b4623d17-6e62-411d-839c-2d768664a920", "395ee0ab-cb3d-48e8-a411-09a98053292f", "18da001e-dd8b-4041-ba4c-eb2211c624c4", "_8i1cwI2bEd6Q36a6kYkaHQ", "df043998-7fd1-4d4d-899f-7257788d644f", "39f96e89-8ead-4742-bc1f-6eb0dd33990b", "_gzH_AI2cEd6Q36a6kYkaHQ", "_SF1a8JY-Ed6rH5SGn-UjSg", "4b01ac5e-7920-491d-af35-1846163e5ad9", "30d03eaa-2615-481c-bf1f-53f177d8147c", "ee920873-7e2d-487f-a51e-76236d6b094e", "a0ceff37-72a6-4da7-a646-89425a748235", "c6ed6258-be4c-4784-a6e4-3e95f9f22dc7"]
}

diagram "[OAIB] Provide Aircraft Localization" {
    view: oab
    shows: ["426f80c2-78ad-4e94-8e5b-da19f22aca26", "f15d0666-1b4a-48d3-a2b1-4206853f1d5a", "45857975-71b5-4d8c-be67-0edb138873d0"]
}

diagram "[OEBD] Operational Entities and Actors" {
    view: oab
    shows: ["d7a95c22-1622-41d5-8201-37cd463075c7", "e1b91d3f-beb2-40fd-869a-ad45ab8ec951", "87bfb1b6-b85e-4497-950d-663dd78c4502", "c4fd192c-6826-4924-bbc3-3a4286b658b4", "dab8526e-cf3a-4c8a-8fa1-7df4aa281bcf", "be0d3f06-3e9b-4162-9ac6-2b16663d1dfa", "dccf58d4-fe45-4a13-bb91-dc1b0c1cc0f5", "2060479f-e59f-4b8f-975f-f05f5986e9b3"]
}

diagram "[OAB] All Operational Activities and Entities" {
    view: oab
    shows: ["2060479f-e59f-4b8f-975f-f05f5986e9b3", "d26f381c-470d-4e9f-a868-cca9bd4a76f6", "f9cabb24-100b-4e68-930a-8f35ec38c106", "45857975-71b5-4d8c-be67-0edb138873d0", "60dc5173-5420-44c0-bedf-ad3f6958a79d", "4f4bb114-4fa9-4b49-aa38-11f7af1e86b1", "e1b91d3f-beb2-40fd-869a-ad45ab8ec951", "87bfb1b6-b85e-4497-950d-663dd78c4502", "280bb458-fb27-41b5-84cf-4af9dd82f2a8", "f4596943-e11d-425c-8b41-4ae491efd471", "7915575a-916b-49ed-a5f0-c782e9135d68", "c4fd192c-6826-4924-bbc3-3a4286b658b4", "41525b3e-aa1f-4002-b245-5e6c0a129fc0", "01a4afcf-b2ef-4562-aa09-6f6fda9f0b93", "dab8526e-cf3a-4c8a-8fa1-7df4aa281bcf", "7f5d8bbc-eca5-4656-a365-757543a38ab3", "cc39bfa2-731b-448d-a8b7-a45d1cf45a97", "426f80c2-78ad-4e94-8e5b-da19f22aca26", "f15d0666-1b4a-48d3-a2b1-4206853f1d5a", "d1d4b34b-685b-4c37-8db4-1d69bb16eb63", "02cf746c-0158-41cf-b626-e3a631132c1e", "dccf58d4-fe45-4a13-bb91-dc1b0c1cc0f5", "4d274ffe-5010-4470-992c-cea2ae1587c8", "dbe67e60-fb6b-4bd1-a85c-039b333e5b0b", "be0d3f06-3e9b-4162-9ac6-2b16663d1dfa", "9902c4e6-e6b7-4b11-9ed8-30c37c8e9426", "7bd0bcf4-2fcc-4229-81f2-8d5cdac49ac8", "d7a95c22-1622-41d5-8201-37cd463075c7", "79a50368-c014-4410-aebd-627426b62614", "1cc1517f-4971-4fbd-be1c-5c83631319a5", "c27f6524-d09e-4962-8f71-c347796bfe6f"]
}

diagram "[OAIB] Watch Movie" {
    view: oab
    shows: ["f9cabb24-100b-4e68-930a-8f35ec38c106", "7f5d8bbc-eca5-4656-a365-757543a38ab3", "7915575a-916b-49ed-a5f0-c782e9135d68"]
}

diagram "[OAIB] Listen to Audio" {
    view: oab
    shows: ["60dc5173-5420-44c0-bedf-ad3f6958a79d", "280bb458-fb27-41b5-84cf-4af9dd82f2a8", "41525b3e-aa1f-4002-b245-5e6c0a129fc0", "cc39bfa2-731b-448d-a8b7-a45d1cf45a97"]
}

diagram "[OABD] All Activities" {
    view: oab
    shows: ["426f80c2-78ad-4e94-8e5b-da19f22aca26", "79dd23d7-af39-4b79-892e-2f8fefbff5ff", "f9cabb24-100b-4e68-930a-8f35ec38c106", "60dc5173-5420-44c0-bedf-ad3f6958a79d", "d26f381c-470d-4e9f-a868-cca9bd4a76f6", "45857975-71b5-4d8c-be67-0edb138873d0", "4f4bb114-4fa9-4b49-aa38-11f7af1e86b1", "9902c4e6-e6b7-4b11-9ed8-30c37c8e9426", "41525b3e-aa1f-4002-b245-5e6c0a129fc0", "02cf746c-0158-41cf-b626-e3a631132c1e", "7f5d8bbc-eca5-4656-a365-757543a38ab3", "7915575a-916b-49ed-a5f0-c782e9135d68", "280bb458-fb27-41b5-84cf-4af9dd82f2a8", "f15d0666-1b4a-48d3-a2b1-4206853f1d5a", "01a4afcf-b2ef-4562-aa09-6f6fda9f0b93", "7bd0bcf4-2fcc-4229-81f2-8d5cdac49ac8", "1cc1517f-4971-4fbd-be1c-5c83631319a5", "4d274ffe-5010-4470-992c-cea2ae1587c8", "dbe67e60-fb6b-4bd1-a85c-039b333e5b0b", "d1d4b34b-685b-4c37-8db4-1d69bb16eb63", "f4596943-e11d-425c-8b41-4ae491efd471", "cc39bfa2-731b-448d-a8b7-a45d1cf45a97", "79a50368-c014-4410-aebd-627426b62614"]
}

diagram "[OAB] High-Level Expected Activities" {
    view: oab
    shows: ["2060479f-e59f-4b8f-975f-f05f5986e9b3", "d26f381c-470d-4e9f-a868-cca9bd4a76f6", "f9cabb24-100b-4e68-930a-8f35ec38c106", "60dc5173-5420-44c0-bedf-ad3f6958a79d", "45857975-71b5-4d8c-be67-0edb138873d0", "4f4bb114-4fa9-4b49-aa38-11f7af1e86b1", "e1b91d3f-beb2-40fd-869a-ad45ab8ec951", "87bfb1b6-b85e-4497-950d-663dd78c4502", "280bb458-fb27-41b5-84cf-4af9dd82f2a8", "f4596943-e11d-425c-8b41-4ae491efd471", "7915575a-916b-49ed-a5f0-c782e9135d68", "c4fd192c-6826-4924-bbc3-3a4286b658b4", "41525b3e-aa1f-4002-b245-5e6c0a129fc0"]
}

diagram "[OCB] Operational Capabilities" {
    view: oab
    shows: ["2060479f-e59f-4b8f-975f-f05f5986e9b3", "87bfb1b6-b85e-4497-950d-663dd78c4502", "c4fd192c-6826-4924-bbc3-3a4286b658b4", "be0d3f06-3e9b-4162-9ac6-2b16663d1dfa"]
}

diagram "[PCBD] Implementation Components" {
    view: pab
    shows: ["5da29b86-88bd-4ced-9992-031690f87f99", "c8114705-19fa-41b3-b534-a342f70c2e51", "eab59749-a0be-4ff3-9321-0d1e4d2f4451", "a850426c-c425-4c85-85fa-cd85b0c066e5", "ee7ecd3f-bf35-4657-b7bd-1e45a8fc025c", "22cbea03-a250-4ea8-a6b3-63829196feb1", "5ed18efd-1535-4aa3-aaaf-48a3ca867607", "01cf8dac-9d35-43aa-a717-5e740989fdc1", "5f666fd8-8032-45f7-821f-1bccbe117b50", "3e6b5b54-0616-4c9e-a017-86f574e2e98f", "404eeafa-1b20-4162-893c-f0cfc1e836d6", "dcb6cca2-ea15-46f9-824f-530d40b4d5f9", "d7856f65-ee48-4e3a-883a-71bcec485da1", "8d1c0fbf-78f2-4cdf-ac2e-f14c31dd3506", "3cbac439-d663-452b-86b6-4f0c947a0047", "fda6b8f9-bfe8-4c60-8443-9b8a9ee2eeeb", "96cfbcc8-37da-4181-b53f-155f8160db5d", "95ea7294-402d-4783-b6e5-2e405c9f4ccb", "84b3c2cb-d4ea-4cce-88e3-930c325463d4", "0cb2ede5-9ff4-417c-b6c4-c4e4f100ebed", "c1cbb014-c2cd-403b-a9c1-62f438e79c8c", "6d095cde-706f-41c2-91ab-7c5189f358c4", "3d96ae05-f435-49b3-abd1-574551f38704", "44003100-30d6-4894-af55-e2e982a5b921", "78c650a1-7440-4335-bf95-f421abdf2c0f", "2632079f-6a54-406a-a957-011c53b16209", "e93cee23-c12e-43e4-ac6d-6049aeb13a63", "fec9b0d7-7a0b-4386-b1e3-4cccb2c721c3", "f52035f8-64d3-4259-8aef-5ad19157efca", "820e9f56-cac0-4043-ba05-19763e6e3788", "f5da7965-ae21-404f-9407-74efd11234ed", "2bf4958b-9ec6-4427-9af0-1d5eefccd900", "6517229a-5dee-4817-978c-8f8cf1bfd3b5", "37d063af-0e66-4f9b-86fc-1a1aaec44b21", "e7049747-365c-4c2f-b769-f45221b9a05a", "737331c9-23e8-4588-969a-fd15444cafa6", "3cba132d-d6ec-4839-b597-2183f9f998b2", "809ec42b-c1ee-471f-a45b-d26977dc956a", "0c7d06de-bc6a-44d2-8b01-459cbdbbf69a", "ce7ff26b-6729-420e-b6a7-78fb80634d88", "47cb2f3d-0ac5-4053-92d1-1af781f0ae20", "403273f6-e4ae-4eed-90f3-d566ecffb01a", "6e43a453-0966-4c19-93e5-c0a692c86941", "cdc16f53-6eaa-4d74-8d63-73afc483dece", "f559c928-27a8-4198-9b0e-fa26404785cd", "eee15697-a375-4a9d-ba93-7ec5b34bc63c", "3f745bea-11ae-4b8c-8a06-930f2391565e", "33ca7fc8-8bd1-48c3-9f22-3fbd217c2549", "af29b99b-0372-4534-906d-6038ff266deb", "a2bf21fb-902c-4b29-87c3-d3436dedb1db", "647781a2-de41-4fa1-a6e6-3a886a706c4e", "9eef8f0f-1bf8-4e51-a17f-480b3afbfd85", "819adf3f-da1d-4d2b-8faf-3b1d7b99503d", "34420358-2236-4010-8504-5fab9af55c0d", "3de7180a-c2f0-48ea-9a9e-6f6e6849c651", "abc69d28-d5c4-470e-afd1-89daa1cc81d2", "a6815c30-8c2b-4273-bbf1-450bb1df58ed", "ec105112-c640-4de6-bc28-461581a1bace", "d187e156-9b25-4adc-8b97-576853d16e40"]
}

diagram "[PCBD] Behavioural Components" {
    view: pab
    shows: ["5da29b86-88bd-4ced-9992-031690f87f99", "c8114705-19fa-41b3-b534-a342f70c2e51", "eab59749-a0be-4ff3-9321-0d1e4d2f4451", "a850426c-c425-4c85-85fa-cd85b0c066e5", "ee7ecd3f-bf35-4657-b7bd-1e45a8fc025c", "22cbea03-a250-4ea8-a6b3-63829196feb1", "5ed18efd-1535-4aa3-aaaf-48a3ca867607", "01cf8dac-9d35-43aa-a717-5e740989fdc1", "5f666fd8-8032-45f7-821f-1bccbe117b50", "3e6b5b54-0616-4c9e-a017-86f574e2e98f", "404eeafa-1b20-4162-893c-f0cfc1e836d6", "dcb6cca2-ea15-46f9-824f-530d40b4d5f9", "d7856f65-ee48-4e3a-883a-71bcec485da1", "8d1c0fbf-78f2-4cdf-ac2e-f14c31dd3506", "3cbac439-d663-452b-86b6-4f0c947a0047", "fda6b8f9-bfe8-4c60-8443-9b8a9ee2eeeb", "96cfbcc8-37da-4181-b53f-155f8160db5d", "95ea7294-402d-4783-b6e5-2e405c9f4ccb", "84b3c2cb-d4ea-4cce-88e3-930c325463d4", "0cb2ede5-9ff4-417c-b6c4-c4e4f100ebed", "c1cbb014-c2cd-403b-a9c1-62f438e79c8c", "6d095cde-706f-41c2-91ab-7c5189f358c4", "3d96ae05-f435-49b3-abd1-574551f38704", "44003100-30d6-4894-af55-e2e982a5b921", "78c650a1-7440-4335-bf95-f421abdf2c0f", "2632079f-6a54-406a-a957-011c53b16209", "e93cee23-c12e-43e4-ac6d-6049aeb13a63", "fec9b0d7-7a0b-4386-b1e3-4cccb2c721c3", "f52035f8-64d3-4259-8aef-5ad19157efca", "820e9f56-cac0-4043-ba05-19763e6e3788", "f5da7965-ae21-404f-9407-74efd11234ed", "2bf4958b-9ec6-4427-9af0-1d5eefccd900", "6517229a-5dee-4817-978c-8f8cf1bfd3b5", "37d063af-0e66-4f9b-86fc-1a1aaec44b21", "e7049747-365c-4c2f-b769-f45221b9a05a", "737331c9-23e8-4588-969a-fd15444cafa6", "3cba132d-d6ec-4839-b597-2183f9f998b2", "809ec42b-c1ee-471f-a45b-d26977dc956a", "0c7d06de-bc6a-44d2-8b01-459cbdbbf69a", "ce7ff26b-6729-420e-b6a7-78fb80634d88", "47cb2f3d-0ac5-4053-92d1-1af781f0ae20", "403273f6-e4ae-4eed-90f3-d566ecffb01a", "6e43a453-0966-4c19-93e5-c0a692c86941", "cdc16f53-6eaa-4d74-8d63-73afc483dece", "f559c928-27a8-4198-9b0e-fa26404785cd", "eee15697-a375-4a9d-ba93-7ec5b34bc63c", "3f745bea-11ae-4b8c-8a06-930f2391565e", "33ca7fc8-8bd1-48c3-9f22-3fbd217c2549", "af29b99b-0372-4534-906d-6038ff266deb", "a2bf21fb-902c-4b29-87c3-d3436dedb1db", "647781a2-de41-4fa1-a6e6-3a886a706c4e", "9eef8f0f-1bf8-4e51-a17f-480b3afbfd85", "819adf3f-da1d-4d2b-8faf-3b1d7b99503d", "34420358-2236-4010-8504-5fab9af55c0d", "3de7180a-c2f0-48ea-9a9e-6f6e6849c651", "abc69d28-d5c4-470e-afd1-89daa1cc81d2", "a6815c30-8c2b-4273-bbf1-450bb1df58ed", "ec105112-c640-4de6-bc28-461581a1bace", "d187e156-9b25-4adc-8b97-576853d16e40"]
}

diagram "[PAB] Implementation and Behaviour Components" {
    view: pab
    shows: ["a850426c-c425-4c85-85fa-cd85b0c066e5", "c1cbb014-c2cd-403b-a9c1-62f438e79c8c", "3f745bea-11ae-4b8c-8a06-930f2391565e", "33ca7fc8-8bd1-48c3-9f22-3fbd217c2549", "0cb2ede5-9ff4-417c-b6c4-c4e4f100ebed", "eee15697-a375-4a9d-ba93-7ec5b34bc63c", "5ed18efd-1535-4aa3-aaaf-48a3ca867607", "ec105112-c640-4de6-bc28-461581a1bace", "5f666fd8-8032-45f7-821f-1bccbe117b50", "96cfbcc8-37da-4181-b53f-155f8160db5d", "ee7ecd3f-bf35-4657-b7bd-1e45a8fc025c", "22cbea03-a250-4ea8-a6b3-63829196feb1", "6d095cde-706f-41c2-91ab-7c5189f358c4", "647781a2-de41-4fa1-a6e6-3a886a706c4e", "737331c9-23e8-4588-969a-fd15444cafa6", "809ec42b-c1ee-471f-a45b-d26977dc956a", "819adf3f-da1d-4d2b-8faf-3b1d7b99503d", "44003100-30d6-4894-af55-e2e982a5b921", "d187e156-9b25-4adc-8b97-576853d16e40", "3d96ae05-f435-49b3-abd1-574551f38704", "34420358-2236-4010-8504-5fab9af55c0d", "78c650a1-7440-4335-bf95-f421abdf2c0f", "3de7180a-c2f0-48ea-9a9e-6f6e6849c651", "2632079f-6a54-406a-a957-011c53b16209", "e93cee23-c12e-43e4-ac6d-6049aeb13a63", "3cba132d-d6ec-4839-b597-2183f9f998b2", "01cf8dac-9d35-43aa-a717-5e740989fdc1", "fec9b0d7-7a0b-4386-b1e3-4cccb2c721c3", "a6815c30-8c2b-4273-bbf1-450bb1df58ed", "403273f6-e4ae-4eed-90f3-d566ecffb01a", "f52035f8-64d3-4259-8aef-5ad19157efca", "abc69d28-d5c4-470e-afd1-89daa1cc81d2", "3e6b5b54-0616-4c9e-a017-86f574e2e98f", "820e9f56-cac0-4043-ba05-19763e6e3788", "a2bf21fb-902c-4b29-87c3-d3436dedb1db", "af29b99b-0372-4534-906d-6038ff266deb", "ce7ff26b-6729-420e-b6a7-78fb80634d88", "f559c928-27a8-4198-9b0e-fa26404785cd", "cdc16f53-6eaa-4d74-8d63-73afc483dece", "dcb6cca2-ea15-46f9-824f-530d40b4d5f9", "0c7d06de-bc6a-44d2-8b01-459cbdbbf69a", "404eeafa-1b20-4162-893c-f0cfc1e836d6"]
}

diagram "[PAB] Focus on Network Setup, Configuration and Tests" {
    view: pab
    shows: ["3e6b5b54-0616-4c9e-a017-86f574e2e98f", "820e9f56-cac0-4043-ba05-19763e6e3788", "af29b99b-0372-4534-906d-6038ff266deb", "6aa6df97-e744-4b1b-962f-66874bfabbaa", "e323d7b2-2bb2-4ffe-a1b5-265c98befe28", "3fcaa02a-0321-4be3-ab79-b2547d47749b", "ee7ecd3f-bf35-4657-b7bd-1e45a8fc025c", "22cbea03-a250-4ea8-a6b3-63829196feb1", "6d095cde-706f-41c2-91ab-7c5189f358c4", "809ec42b-c1ee-471f-a45b-d26977dc956a", "b2fe1c12-86f0-4c40-9986-d349c2a09496", "84c8b355-ee9b-48dd-aca8-17b0c4e477c8", "01cf8dac-9d35-43aa-a717-5e740989fdc1", "fec9b0d7-7a0b-4386-b1e3-4cccb2c721c3", "403273f6-e4ae-4eed-90f3-d566ecffb01a", "c5503a00-fae8-439d-90a7-6b8644d2f609", "238797f1-c012-446a-9b79-e94cf88bd180"]
}

diagram "REC - Unit Network Adapter" {
    view: pab
    shows: ["6e43a453-0966-4c19-93e5-c0a692c86941", "ccfb7dc0-c861-44db-95ea-0a216b9ec643", "e24662c4-47b2-445f-9d18-7214fca7013e"]
}

diagram "RPL - Instantiations of Unit Network Adapter" {
    view: pab
    shows: ["403273f6-e4ae-4eed-90f3-d566ecffb01a", "c5503a00-fae8-439d-90a7-6b8644d2f609", "238797f1-c012-446a-9b79-e94cf88bd180", "809ec42b-c1ee-471f-a45b-d26977dc956a", "84c8b355-ee9b-48dd-aca8-17b0c4e477c8", "b2fe1c12-86f0-4c40-9986-d349c2a09496", "33ca7fc8-8bd1-48c3-9f22-3fbd217c2549", "aecf56f9-9069-4d1d-87c5-3cd5fbd7e4fe", "c4a29500-aeda-49ed-be51-65ca064266b6"]
}

diagram "[PAB] [BUILD] All PCs, PFs, FEs" {
    view: pab
    shows: ["5ed18efd-1535-4aa3-aaaf-48a3ca867607", "ec105112-c640-4de6-bc28-461581a1bace", "0c8009a9-0524-4445-a9c8-f0295467f62a", "ee7ecd3f-bf35-4657-b7bd-1e45a8fc025c", "22cbea03-a250-4ea8-a6b3-63829196feb1", "6d095cde-706f-41c2-91ab-7c5189f358c4", "737331c9-23e8-4588-969a-fd15444cafa6", "8835c15a-0eb0-4d20-83d2-a15f8c0bd005", "3f69f8bd-704d-438b-8f05-fc7d56363811", "647781a2-de41-4fa1-a6e6-3a886a706c4e", "ac5be58c-aa92-4247-8c8f-9867ddbd6289", "dcbf11ee-150a-4afa-8e06-3f0ab9aee27a", "8cc1e6f5-dbe2-4629-8a43-b60eef771a8f", "241dcfae-1880-45f9-bf2c-32b0cbe736b7", "819adf3f-da1d-4d2b-8faf-3b1d7b99503d", "8daae3ab-944b-4ade-9896-0d51c62ef3e8", "4c361e00-d6d0-4acf-a523-1de11acf7797", "1289a361-a3e8-416b-a9e0-e4fcd528d9e3", "44003100-30d6-4894-af55-e2e982a5b921", "d187e156-9b25-4adc-8b97-576853d16e40", "bd383ff2-258a-41dd-973f-07f7aa30da7a", "83b4863a-7795-4911-aeef-2938443cce3d", "068dc2db-3e29-421d-84be-8444ce8acc9c", "c7edaab3-11bc-4409-b300-13c4b4192742", "3d96ae05-f435-49b3-abd1-574551f38704", "34420358-2236-4010-8504-5fab9af55c0d", "ecc7d466-f9cb-4be1-a099-b8449146cf18", "9d3a7965-d967-42e2-8f25-c0113abaf9ff", "78c650a1-7440-4335-bf95-f421abdf2c0f", "3de7180a-c2f0-48ea-9a9e-6f6e6849c651", "9155e7ae-b4c3-4f06-b8b2-0ec90045ac8e", "07823fa4-38c9-43fe-8b4b-8aa5588b7c46", "e93cee23-c12e-43e4-ac6d-6049aeb13a63", "3cba132d-d6ec-4839-b597-2183f9f998b2", "a84dff3b-2158-41ed-b96c-0b8ace0bea3b", "ca1519c5-e748-43ee-a81b-474a31b7b5ab", "a3605148-c80e-4635-9790-f068451a1b4a", "be579285-807c-46ab-bcab-ae7efb19a8d6", "649d218e-6119-46b3-8c9e-377cf40e086d", "643033c9-5624-45f8-a1ce-b9bc4ddc2553", "3159e079-92f0-4e22-8cef-53e8f447cd5f", "a850426c-c425-4c85-85fa-cd85b0c066e5", "0cb2ede5-9ff4-417c-b6c4-c4e4f100ebed", "eee15697-a375-4a9d-ba93-7ec5b34bc63c", "0378eba1-fb7b-4b4d-b39d-7a14ce4e5f4e", "c1cbb014-c2cd-403b-a9c1-62f438e79c8c", "3f745bea-11ae-4b8c-8a06-930f2391565e", "2dfb0a52-7568-4903-9c78-1e462848f868", "98960be5-3e8f-4743-ab70-805645472512", "d53e6dd4-3926-4b25-80d1-14553a472f9e", "3e6b5b54-0616-4c9e-a017-86f574e2e98f", "820e9f56-cac0-4043-ba05-19763e6e3788", "a2bf21fb-902c-4b29-87c3-d3436dedb1db", "80a8d0f6-ca9e-486f-9ac3-0c524de61281", "88a4d6d0-fcf2-4c03-be5e-e6a23c727bf4", "ce7ff26b-6729-420e-b6a7-78fb80634d88", "28b23885-f94c-44b9-8c2f-a2f52b379842", "c91204f5-d5ed-401e-b83d-d6e85d7d887c", "cdc16f53-6eaa-4d74-8d63-73afc483dece", "b4c36894-10af-4166-9da7-099de70ad0f6", "2788898f-6daf-43c1-bca8-6ab484ac24c6", "ccf41abe-2a9d-4f50-bc3d-643397b7a338", "f559c928-27a8-4198-9b0e-fa26404785cd", "c13d6970-0ca4-44b4-be4b-64b6f3eebbb4", "404eeafa-1b20-4162-893c-f0cfc1e836d6", "01cf8dac-9d35-43aa-a717-5e740989fdc1", "f52035f8-64d3-4259-8aef-5ad19157efca", "abc69d28-d5c4-470e-afd1-89daa1cc81d2", "72f0a4da-866e-4136-8ca7-bfc85db05005", "cebc5976-81d6-4718-b1f0-9f8b7687ceeb", "9b798c7d-2d5a-43f9-867d-b26c3f4c1337", "fec9b0d7-7a0b-4386-b1e3-4cccb2c721c3", "a6815c30-8c2b-4273-bbf1-450bb1df58ed", "e8af73d3-bacb-4222-b387-f6b0bd4d13a1", "737aebcf-1f00-40ac-8e03-df8bc6596159", "1cda3b05-ee17-4912-b875-40bc4ddca86b", "5f666fd8-8032-45f7-821f-1bccbe117b50", "96cfbcc8-37da-4181-b53f-155f8160db5d", "16904687-4018-46cc-a0ec-c415f6d2bc1d", "dcb6cca2-ea15-46f9-824f-530d40b4d5f9", "0c7d06de-bc6a-44d2-8b01-459cbdbbf69a", "23b2867d-b9cd-4b1d-8d40-7de838fc1313", "8aba6e60-54e0-401a-bc8b-c86090e030d0", "9c247f2d-ed9b-43e8-921a-d1fdbb331ab1", "820d4407-7206-4075-91fc-d76ce083056e", "39b3d893-7961-4f0e-9b35-6090100f2c37", "6a02a1d5-5b6b-4ea7-8494-4afca0a68e4e", "75e214e4-74f8-4c44-97f9-f328b8ac3f97", "4c237128-0cf3-4074-8af4-5e63e4994e02", "921af19b-4f01-46af-b01e-f6cd6bbfe3c1", "a5782ec2-f546-49e5-98aa-88146d997d1d", "9b00bf4f-2f71-4f7a-9c51-4a84b84e18de", "292159f9-5917-4c0c-9ea3-2800431b3c48", "f459f10a-d8ae-4938-b3e1-4eb294ec1c32", "a8b9e953-b751-4991-b90e-e3ec251f65f0"]
}

diagram "[PFBD] All Physical Functions" {
    view: pab
    shows: ["e22a5c1a-1ee1-4bc4-8b34-b843f4bcf116", "292159f9-5917-4c0c-9ea3-2800431b3c48", "9b00bf4f-2f71-4f7a-9c51-4a84b84e18de", "07256f31-32c7-42f5-b36b-bebd43492921", "be579285-807c-46ab-bcab-ae7efb19a8d6", "649d218e-6119-46b3-8c9e-377cf40e086d", "a3605148-c80e-4635-9790-f068451a1b4a", "643033c9-5624-45f8-a1ce-b9bc4ddc2553", "3159e079-92f0-4e22-8cef-53e8f447cd5f", "75c66998-1426-4629-94e6-b01fafd37579", "921af19b-4f01-46af-b01e-f6cd6bbfe3c1", "39b3d893-7961-4f0e-9b35-6090100f2c37", "4c237128-0cf3-4074-8af4-5e63e4994e02", "6a02a1d5-5b6b-4ea7-8494-4afca0a68e4e", "820d4407-7206-4075-91fc-d76ce083056e", "9c247f2d-ed9b-43e8-921a-d1fdbb331ab1", "75e214e4-74f8-4c44-97f9-f328b8ac3f97", "6093275a-6ff1-49a6-9384-dea2bab8510d", "eab9aed9-2576-4d7b-ad86-021e79aac4e1", "03df466f-aebe-4fec-8617-e82e6bf9ed5d", "d8b3f902-a4ed-4c9e-a650-e5c159c8d74f", "a8b9e953-b751-4991-b90e-e3ec251f65f0", "c75359fb-2271-4499-8199-c3e1565697a7", "1da11685-85e8-4ea3-a6e8-e7cfe33111d4", "b4c36894-10af-4166-9da7-099de70ad0f6", "2788898f-6daf-43c1-bca8-6ab484ac24c6", "54b31979-1ec1-49c5-81e0-512fd1aec704", "16904687-4018-46cc-a0ec-c415f6d2bc1d", "0378eba1-fb7b-4b4d-b39d-7a14ce4e5f4e", "c13d6970-0ca4-44b4-be4b-64b6f3eebbb4", "03b0cf66-8fd1-4867-bec3-bc49814211a9", "7f4ecc96-3b56-4470-a35b-ce764d0b4e1f", "3fcaa02a-0321-4be3-ab79-b2547d47749b", "e323d7b2-2bb2-4ffe-a1b5-265c98befe28", "6aa6df97-e744-4b1b-962f-66874bfabbaa", "c5503a00-fae8-439d-90a7-6b8644d2f609", "238797f1-c012-446a-9b79-e94cf88bd180", "b2fe1c12-86f0-4c40-9986-d349c2a09496", "84c8b355-ee9b-48dd-aca8-17b0c4e477c8", "aecf56f9-9069-4d1d-87c5-3cd5fbd7e4fe", "c4a29500-aeda-49ed-be51-65ca064266b6", "cb0426b2-9cab-4a4d-9e7b-89a13a57be2e", "8e79917f-4323-4040-b8c5-64c5e42c2a3b", "ac5be58c-aa92-4247-8c8f-9867ddbd6289", "737aebcf-1f00-40ac-8e03-df8bc6596159", "cf0384ae-550f-40a4-a860-7d081b24a01e", "d53e6dd4-3926-4b25-80d1-14553a472f9e", "98960be5-3e8f-4743-ab70-805645472512", "2dfb0a52-7568-4903-9c78-1e462848f868", "3d159790-b7eb-46d7-90ac-d64017225cf0", "d8e4de58-aebd-43de-85d6-b7e0011cf619", "ecc7d466-f9cb-4be1-a099-b8449146cf18", "9d3a7965-d967-42e2-8f25-c0113abaf9ff", "880b2d92-4a5f-4b0c-9c54-4faa45efd4aa", "28b23885-f94c-44b9-8c2f-a2f52b379842", "c91204f5-d5ed-401e-b83d-d6e85d7d887c", "e8af73d3-bacb-4222-b387-f6b0bd4d13a1", "72f0a4da-866e-4136-8ca7-bfc85db05005", "068dc2db-3e29-421d-84be-8444ce8acc9c", "6f47362b-60ef-451f-8a00-005ebd4fd010", "9155e7ae-b4c3-4f06-b8b2-0ec90045ac8e", "07823fa4-38c9-43fe-8b4b-8aa5588b7c46", "bd383ff2-258a-41dd-973f-07f7aa30da7a", "23b2867d-b9cd-4b1d-8d40-7de838fc1313", "4c361e00-d6d0-4acf-a523-1de11acf7797", "a84dff3b-2158-41ed-b96c-0b8ace0bea3b", "ccf41abe-2a9d-4f50-bc3d-643397b7a338", "1289a361-a3e8-416b-a9e0-e4fcd528d9e3", "80a8d0f6-ca9e-486f-9ac3-0c524de61281", "196af4c5-87f3-44b7-8cc6-74d91d4d1c9a", "f9353538-3fcb-4697-8591-daf861b7a164", "c7edaab3-11bc-4409-b300-13c4b4192742", "0c8009a9-0524-4445-a9c8-f0295467f62a", "dcbf11ee-150a-4afa-8e06-3f0ab9aee27a", "241dcfae-1880-45f9-bf2c-32b0cbe736b7", "8daae3ab-944b-4ade-9896-0d51c62ef3e8", "46a33304-1338-4fc0-a9cb-dc3fe19e5ed4", "9b798c7d-2d5a-43f9-867d-b26c3f4c1337", "1cda3b05-ee17-4912-b875-40bc4ddca86b", "cebc5976-81d6-4718-b1f0-9f8b7687ceeb", "cf7dca25-2250-4339-9501-eaf3557a644c", "83b4863a-7795-4911-aeef-2938443cce3d", "9d76fefa-2f89-4272-8b1d-785d05601ce6", "2e06d355-e036-40f8-9f84-074d413e7b07", "8835c15a-0eb0-4d20-83d2-a15f8c0bd005", "8cc1e6f5-dbe2-4629-8a43-b60eef771a8f", "88a4d6d0-fcf2-4c03-be5e-e6a23c727bf4", "3f69f8bd-704d-438b-8f05-fc7d56363811", "f46f4ddc-51cc-4b2c-ba6c-d9e8c0bae8ea", "88688938-d911-4317-b36e-266cef11a7f3", "fd6de739-ae93-4e5f-acaf-4e8fb8f0b2e5", "1c49d7b2-bb7a-4939-86c1-a73c0822fdf5", "79218945-69c6-4bc6-8aa6-9d67f537492d", "0012d480-7be3-452f-9872-3a0d8868e2f6", "1291c3c4-170d-4fcc-9b59-5b0961cd4a2d", "71147664-0fcd-401f-937a-bdc76216924d", "62f152bd-6017-4828-9034-7af3c387f83a", "29c4014e-7db7-4660-9f3e-e0a8a37e0090", "52d67cae-5ad2-4c5d-b4bf-e68d308c73b3", "71f18377-d79b-4619-a537-cfc07ecb8931", "e24662c4-47b2-445f-9d18-7214fca7013e", "ccfb7dc0-c861-44db-95ea-0a216b9ec643"]
}

diagram "[PDFB] [CTX] Manage Audio Video Diffusion" {
    view: pab
    shows: ["cb0426b2-9cab-4a4d-9e7b-89a13a57be2e", "cf0384ae-550f-40a4-a860-7d081b24a01e", "3d159790-b7eb-46d7-90ac-d64017225cf0", "8e79917f-4323-4040-b8c5-64c5e42c2a3b", "80a8d0f6-ca9e-486f-9ac3-0c524de61281", "196af4c5-87f3-44b7-8cc6-74d91d4d1c9a", "0378eba1-fb7b-4b4d-b39d-7a14ce4e5f4e", "292159f9-5917-4c0c-9ea3-2800431b3c48", "be579285-807c-46ab-bcab-ae7efb19a8d6", "649d218e-6119-46b3-8c9e-377cf40e086d", "3159e079-92f0-4e22-8cef-53e8f447cd5f", "1da11685-85e8-4ea3-a6e8-e7cfe33111d4", "cf7dca25-2250-4339-9501-eaf3557a644c", "9c247f2d-ed9b-43e8-921a-d1fdbb331ab1"]
}

diagram "[PDFB] [CTX] Broadcast Stored Audio and Video Streams" {
    view: pab
    shows: ["cf0384ae-550f-40a4-a860-7d081b24a01e", "d53e6dd4-3926-4b25-80d1-14553a472f9e", "98960be5-3e8f-4743-ab70-805645472512", "2dfb0a52-7568-4903-9c78-1e462848f868", "3d159790-b7eb-46d7-90ac-d64017225cf0", "8e79917f-4323-4040-b8c5-64c5e42c2a3b", "0378eba1-fb7b-4b4d-b39d-7a14ce4e5f4e"]
}

diagram "[PDFB] [CTX] Display Video and Play Audio" {
    view: pab
    shows: ["3d159790-b7eb-46d7-90ac-d64017225cf0", "880b2d92-4a5f-4b0c-9c54-4faa45efd4aa", "d8e4de58-aebd-43de-85d6-b7e0011cf619", "e8af73d3-bacb-4222-b387-f6b0bd4d13a1", "72f0a4da-866e-4136-8ca7-bfc85db05005", "068dc2db-3e29-421d-84be-8444ce8acc9c", "6f47362b-60ef-451f-8a00-005ebd4fd010", "bd383ff2-258a-41dd-973f-07f7aa30da7a", "23b2867d-b9cd-4b1d-8d40-7de838fc1313", "4c361e00-d6d0-4acf-a523-1de11acf7797", "a84dff3b-2158-41ed-b96c-0b8ace0bea3b", "ccf41abe-2a9d-4f50-bc3d-643397b7a338", "1289a361-a3e8-416b-a9e0-e4fcd528d9e3", "cf0384ae-550f-40a4-a860-7d081b24a01e", "be579285-807c-46ab-bcab-ae7efb19a8d6", "649d218e-6119-46b3-8c9e-377cf40e086d", "3159e079-92f0-4e22-8cef-53e8f447cd5f", "292159f9-5917-4c0c-9ea3-2800431b3c48", "80a8d0f6-ca9e-486f-9ac3-0c524de61281", "cf7dca25-2250-4339-9501-eaf3557a644c", "9c247f2d-ed9b-43e8-921a-d1fdbb331ab1"]
}

diagram "[PDFB] [CTX] Play Video Stream on Seat TV" {
    view: pab
    shows: ["d8e4de58-aebd-43de-85d6-b7e0011cf619", "ecc7d466-f9cb-4be1-a099-b8449146cf18", "9d3a7965-d967-42e2-8f25-c0113abaf9ff", "cf0384ae-550f-40a4-a860-7d081b24a01e", "bd383ff2-258a-41dd-973f-07f7aa30da7a", "4c361e00-d6d0-4acf-a523-1de11acf7797", "1289a361-a3e8-416b-a9e0-e4fcd528d9e3"]
}

diagram "[PAB] [BUILD] Template" {
    view: pab
    shows: ["5ed18efd-1535-4aa3-aaaf-48a3ca867607", "ec105112-c640-4de6-bc28-461581a1bace", "0c8009a9-0524-4445-a9c8-f0295467f62a", "9d76fefa-2f89-4272-8b1d-785d05601ce6", "ee7ecd3f-bf35-4657-b7bd-1e45a8fc025c", "22cbea03-a250-4ea8-a6b3-63829196feb1", "6d095cde-706f-41c2-91ab-7c5189f358c4", "737331c9-23e8-4588-969a-fd15444cafa6", "8835c15a-0eb0-4d20-83d2-a15f8c0bd005", "3f69f8bd-704d-438b-8f05-fc7d56363811", "647781a2-de41-4fa1-a6e6-3a886a706c4e", "ac5be58c-aa92-4247-8c8f-9867ddbd6289", "dcbf11ee-150a-4afa-8e06-3f0ab9aee27a", "241dcfae-1880-45f9-bf2c-32b0cbe736b7", "8cc1e6f5-dbe2-4629-8a43-b60eef771a8f", "819adf3f-da1d-4d2b-8faf-3b1d7b99503d", "8daae3ab-944b-4ade-9896-0d51c62ef3e8", "4c361e00-d6d0-4acf-a523-1de11acf7797", "1289a361-a3e8-416b-a9e0-e4fcd528d9e3", "809ec42b-c1ee-471f-a45b-d26977dc956a", "b2fe1c12-86f0-4c40-9986-d349c2a09496", "84c8b355-ee9b-48dd-aca8-17b0c4e477c8", "44003100-30d6-4894-af55-e2e982a5b921", "d187e156-9b25-4adc-8b97-576853d16e40", "068dc2db-3e29-421d-84be-8444ce8acc9c", "c7edaab3-11bc-4409-b300-13c4b4192742", "83b4863a-7795-4911-aeef-2938443cce3d", "bd383ff2-258a-41dd-973f-07f7aa30da7a", "3d96ae05-f435-49b3-abd1-574551f38704", "34420358-2236-4010-8504-5fab9af55c0d", "ecc7d466-f9cb-4be1-a099-b8449146cf18", "9d3a7965-d967-42e2-8f25-c0113abaf9ff", "78c650a1-7440-4335-bf95-f421abdf2c0f", "3de7180a-c2f0-48ea-9a9e-6f6e6849c651", "07823fa4-38c9-43fe-8b4b-8aa5588b7c46", "9155e7ae-b4c3-4f06-b8b2-0ec90045ac8e", "e93cee23-c12e-43e4-ac6d-6049aeb13a63", "3cba132d-d6ec-4839-b597-2183f9f998b2", "a84dff3b-2158-41ed-b96c-0b8ace0bea3b", "2632079f-6a54-406a-a957-011c53b16209", "ca1519c5-e748-43ee-a81b-474a31b7b5ab", "be579285-807c-46ab-bcab-ae7efb19a8d6", "649d218e-6119-46b3-8c9e-377cf40e086d", "643033c9-5624-45f8-a1ce-b9bc4ddc2553", "a3605148-c80e-4635-9790-f068451a1b4a", "3159e079-92f0-4e22-8cef-53e8f447cd5f", "a850426c-c425-4c85-85fa-cd85b0c066e5", "0cb2ede5-9ff4-417c-b6c4-c4e4f100ebed", "eee15697-a375-4a9d-ba93-7ec5b34bc63c", "0378eba1-fb7b-4b4d-b39d-7a14ce4e5f4e", "c1cbb014-c2cd-403b-a9c1-62f438e79c8c", "3f745bea-11ae-4b8c-8a06-930f2391565e", "98960be5-3e8f-4743-ab70-805645472512", "d53e6dd4-3926-4b25-80d1-14553a472f9e", "2dfb0a52-7568-4903-9c78-1e462848f868", "3e6b5b54-0616-4c9e-a017-86f574e2e98f", "820e9f56-cac0-4043-ba05-19763e6e3788", "a2bf21fb-902c-4b29-87c3-d3436dedb1db", "80a8d0f6-ca9e-486f-9ac3-0c524de61281", "88a4d6d0-fcf2-4c03-be5e-e6a23c727bf4", "ce7ff26b-6729-420e-b6a7-78fb80634d88", "c91204f5-d5ed-401e-b83d-d6e85d7d887c", "28b23885-f94c-44b9-8c2f-a2f52b379842", "f559c928-27a8-4198-9b0e-fa26404785cd", "c13d6970-0ca4-44b4-be4b-64b6f3eebbb4", "cdc16f53-6eaa-4d74-8d63-73afc483dece", "ccf41abe-2a9d-4f50-bc3d-643397b7a338", "2788898f-6daf-43c1-bca8-6ab484ac24c6", "b4c36894-10af-4166-9da7-099de70ad0f6", "af29b99b-0372-4534-906d-6038ff266deb", "6aa6df97-e744-4b1b-962f-66874bfabbaa", "e323d7b2-2bb2-4ffe-a1b5-265c98befe28", "3fcaa02a-0321-4be3-ab79-b2547d47749b", "404eeafa-1b20-4162-893c-f0cfc1e836d6", "01cf8dac-9d35-43aa-a717-5e740989fdc1", "f52035f8-64d3-4259-8aef-5ad19157efca", "abc69d28-d5c4-470e-afd1-89daa1cc81d2", "9b798c7d-2d5a-43f9-867d-b26c3f4c1337", "72f0a4da-866e-4136-8ca7-bfc85db05005", "cebc5976-81d6-4718-b1f0-9f8b7687ceeb", "fec9b0d7-7a0b-4386-b1e3-4cccb2c721c3", "a6815c30-8c2b-4273-bbf1-450bb1df58ed", "e8af73d3-bacb-4222-b387-f6b0bd4d13a1", "737aebcf-1f00-40ac-8e03-df8bc6596159", "1cda3b05-ee17-4912-b875-40bc4ddca86b", "403273f6-e4ae-4eed-90f3-d566ecffb01a", "238797f1-c012-446a-9b79-e94cf88bd180", "c5503a00-fae8-439d-90a7-6b8644d2f609", "5f666fd8-8032-45f7-821f-1bccbe117b50", "96cfbcc8-37da-4181-b53f-155f8160db5d", "16904687-4018-46cc-a0ec-c415f6d2bc1d", "dcb6cca2-ea15-46f9-824f-530d40b4d5f9", "0c7d06de-bc6a-44d2-8b01-459cbdbbf69a", "23b2867d-b9cd-4b1d-8d40-7de838fc1313", "8aba6e60-54e0-401a-bc8b-c86090e030d0", "39b3d893-7961-4f0e-9b35-6090100f2c37", "921af19b-4f01-46af-b01e-f6cd6bbfe3c1", "9c247f2d-ed9b-43e8-921a-d1fdbb331ab1", "75e214e4-74f8-4c44-97f9-f328b8ac3f97", "6a02a1d5-5b6b-4ea7-8494-4afca0a68e4e", "4c237128-0cf3-4074-8af4-5e63e4994e02", "820d4407-7206-4075-91fc-d76ce083056e", "a5782ec2-f546-49e5-98aa-88146d997d1d", "9b00bf4f-2f71-4f7a-9c51-4a84b84e18de", "292159f9-5917-4c0c-9ea3-2800431b3c48", "f459f10a-d8ae-4938-b3e1-4eb294ec1c32", "eab9aed9-2576-4d7b-ad86-021e79aac4e1", "d8b3f902-a4ed-4c9e-a650-e5c159c8d74f", "03df466f-aebe-4fec-8617-e82e6bf9ed5d", "a8b9e953-b751-4991-b90e-e3ec251f65f0"]
}

diagram "[PDFB] [CTX] Process Audio Video Requests" {
    view: pab
    shows: ["8e79917f-4323-4040-b8c5-64c5e42c2a3b", "ac5be58c-aa92-4247-8c8f-9867ddbd6289", "737aebcf-1f00-40ac-8e03-df8bc6596159", "cf0384ae-550f-40a4-a860-7d081b24a01e", "196af4c5-87f3-44b7-8cc6-74d91d4d1c9a", "cf7dca25-2250-4339-9501-eaf3557a644c"]
}

diagram "[PDFB] [CTX] Run Services" {
    view: pab
    shows: ["196af4c5-87f3-44b7-8cc6-74d91d4d1c9a", "f9353538-3fcb-4697-8591-daf861b7a164", "46a33304-1338-4fc0-a9cb-dc3fe19e5ed4", "e22a5c1a-1ee1-4bc4-8b34-b843f4bcf116", "07256f31-32c7-42f5-b36b-bebd43492921", "cf7dca25-2250-4339-9501-eaf3557a644c", "54b31979-1ec1-49c5-81e0-512fd1aec704", "cb0426b2-9cab-4a4d-9e7b-89a13a57be2e"]
}

diagram "[PDFB] [CTX] Manage Passenger Services Lifecycle" {
    view: pab
    shows: ["cf7dca25-2250-4339-9501-eaf3557a644c", "83b4863a-7795-4911-aeef-2938443cce3d", "9d76fefa-2f89-4272-8b1d-785d05601ce6", "88a4d6d0-fcf2-4c03-be5e-e6a23c727bf4", "8835c15a-0eb0-4d20-83d2-a15f8c0bd005", "8cc1e6f5-dbe2-4629-8a43-b60eef771a8f", "3f69f8bd-704d-438b-8f05-fc7d56363811", "a3605148-c80e-4635-9790-f068451a1b4a", "a8b9e953-b751-4991-b90e-e3ec251f65f0", "c13d6970-0ca4-44b4-be4b-64b6f3eebbb4", "196af4c5-87f3-44b7-8cc6-74d91d4d1c9a", "80a8d0f6-ca9e-486f-9ac3-0c524de61281", "068dc2db-3e29-421d-84be-8444ce8acc9c", "737aebcf-1f00-40ac-8e03-df8bc6596159"]
}

diagram "[PDFB] [CTX] Run Video-On-Demand Service" {
    view: pab
    shows: ["f9353538-3fcb-4697-8591-daf861b7a164", "0c8009a9-0524-4445-a9c8-f0295467f62a", "c7edaab3-11bc-4409-b300-13c4b4192742", "dcbf11ee-150a-4afa-8e06-3f0ab9aee27a", "241dcfae-1880-45f9-bf2c-32b0cbe736b7", "8daae3ab-944b-4ade-9896-0d51c62ef3e8", "0378eba1-fb7b-4b4d-b39d-7a14ce4e5f4e", "cb0426b2-9cab-4a4d-9e7b-89a13a57be2e", "cf7dca25-2250-4339-9501-eaf3557a644c", "643033c9-5624-45f8-a1ce-b9bc4ddc2553"]
}

diagram "[PAB] [CTX] Watch Imposed Video on Cabin Screen FC" {
    view: pab
    shows: ["ca1519c5-e748-43ee-a81b-474a31b7b5ab", "649d218e-6119-46b3-8c9e-377cf40e086d", "a850426c-c425-4c85-85fa-cd85b0c066e5", "0cb2ede5-9ff4-417c-b6c4-c4e4f100ebed", "eee15697-a375-4a9d-ba93-7ec5b34bc63c", "0378eba1-fb7b-4b4d-b39d-7a14ce4e5f4e", "c1cbb014-c2cd-403b-a9c1-62f438e79c8c", "3f745bea-11ae-4b8c-8a06-930f2391565e", "d53e6dd4-3926-4b25-80d1-14553a472f9e", "2dfb0a52-7568-4903-9c78-1e462848f868", "98960be5-3e8f-4743-ab70-805645472512", "3e6b5b54-0616-4c9e-a017-86f574e2e98f", "820e9f56-cac0-4043-ba05-19763e6e3788", "ce7ff26b-6729-420e-b6a7-78fb80634d88", "28b23885-f94c-44b9-8c2f-a2f52b379842", "c91204f5-d5ed-401e-b83d-d6e85d7d887c", "cdc16f53-6eaa-4d74-8d63-73afc483dece", "ccf41abe-2a9d-4f50-bc3d-643397b7a338", "404eeafa-1b20-4162-893c-f0cfc1e836d6", "01cf8dac-9d35-43aa-a717-5e740989fdc1", "f52035f8-64d3-4259-8aef-5ad19157efca", "abc69d28-d5c4-470e-afd1-89daa1cc81d2", "9b798c7d-2d5a-43f9-867d-b26c3f4c1337", "72f0a4da-866e-4136-8ca7-bfc85db05005", "fec9b0d7-7a0b-4386-b1e3-4cccb2c721c3", "a6815c30-8c2b-4273-bbf1-450bb1df58ed", "e8af73d3-bacb-4222-b387-f6b0bd4d13a1", "737aebcf-1f00-40ac-8e03-df8bc6596159", "dcb6cca2-ea15-46f9-824f-530d40b4d5f9", "0c7d06de-bc6a-44d2-8b01-459cbdbbf69a", "23b2867d-b9cd-4b1d-8d40-7de838fc1313", "8aba6e60-54e0-401a-bc8b-c86090e030d0", "9c247f2d-ed9b-43e8-921a-d1fdbb331ab1", "a5782ec2-f546-49e5-98aa-88146d997d1d", "292159f9-5917-4c0c-9ea3-2800431b3c48"]
}

diagram "[PDFB] [CTX] Play Video Stream on Cabin Screen" {
    view: pab
    shows: ["880b2d92-4a5f-4b0c-9c54-4faa45efd4aa", "c91204f5-d5ed-401e-b83d-d6e85d7d887c", "28b23885-f94c-44b9-8c2f-a2f52b379842", "cf0384ae-550f-40a4-a860-7d081b24a01e", "23b2867d-b9cd-4b1d-8d40-7de838fc1313"]
}

diagram "[PAB] [CTX] Start Playing VOD Movie FC" {
    view: pab
    shows: ["5ed18efd-1535-4aa3-aaaf-48a3ca867607", "ec105112-c640-4de6-bc28-461581a1bace", "0c8009a9-0524-4445-a9c8-f0295467f62a", "22cbea03-a250-4ea8-a6b3-63829196feb1", "6d095cde-706f-41c2-91ab-7c5189f358c4", "647781a2-de41-4fa1-a6e6-3a886a706c4e", "ac5be58c-aa92-4247-8c8f-9867ddbd6289", "819adf3f-da1d-4d2b-8faf-3b1d7b99503d", "4c361e00-d6d0-4acf-a523-1de11acf7797", "1289a361-a3e8-416b-a9e0-e4fcd528d9e3", "44003100-30d6-4894-af55-e2e982a5b921", "d187e156-9b25-4adc-8b97-576853d16e40", "bd383ff2-258a-41dd-973f-07f7aa30da7a", "3d96ae05-f435-49b3-abd1-574551f38704", "34420358-2236-4010-8504-5fab9af55c0d", "9d3a7965-d967-42e2-8f25-c0113abaf9ff", "ecc7d466-f9cb-4be1-a099-b8449146cf18", "78c650a1-7440-4335-bf95-f421abdf2c0f", "3de7180a-c2f0-48ea-9a9e-6f6e6849c651", "9155e7ae-b4c3-4f06-b8b2-0ec90045ac8e", "07823fa4-38c9-43fe-8b4b-8aa5588b7c46", "e93cee23-c12e-43e4-ac6d-6049aeb13a63", "3cba132d-d6ec-4839-b597-2183f9f998b2", "a84dff3b-2158-41ed-b96c-0b8ace0bea3b", "ca1519c5-e748-43ee-a81b-474a31b7b5ab", "be579285-807c-46ab-bcab-ae7efb19a8d6", "643033c9-5624-45f8-a1ce-b9bc4ddc2553", "3159e079-92f0-4e22-8cef-53e8f447cd5f", "a850426c-c425-4c85-85fa-cd85b0c066e5", "0cb2ede5-9ff4-417c-b6c4-c4e4f100ebed", "eee15697-a375-4a9d-ba93-7ec5b34bc63c", "0378eba1-fb7b-4b4d-b39d-7a14ce4e5f4e", "c1cbb014-c2cd-403b-a9c1-62f438e79c8c", "3f745bea-11ae-4b8c-8a06-930f2391565e", "d53e6dd4-3926-4b25-80d1-14553a472f9e", "98960be5-3e8f-4743-ab70-805645472512", "2dfb0a52-7568-4903-9c78-1e462848f868"]
}

diagram "[PDFB] [CTX] Play Audio on Seat TV" {
    view: pab
    shows: ["6f47362b-60ef-451f-8a00-005ebd4fd010", "9155e7ae-b4c3-4f06-b8b2-0ec90045ac8e", "07823fa4-38c9-43fe-8b4b-8aa5588b7c46", "80a8d0f6-ca9e-486f-9ac3-0c524de61281", "4c361e00-d6d0-4acf-a523-1de11acf7797", "a84dff3b-2158-41ed-b96c-0b8ace0bea3b", "1289a361-a3e8-416b-a9e0-e4fcd528d9e3"]
}

diagram "[PAB] Focus on SVDU Audio Video Playing" {
    view: pab
    shows: ["22cbea03-a250-4ea8-a6b3-63829196feb1", "6d095cde-706f-41c2-91ab-7c5189f358c4", "819adf3f-da1d-4d2b-8faf-3b1d7b99503d", "8daae3ab-944b-4ade-9896-0d51c62ef3e8", "4c361e00-d6d0-4acf-a523-1de11acf7797", "1289a361-a3e8-416b-a9e0-e4fcd528d9e3", "44003100-30d6-4894-af55-e2e982a5b921", "d187e156-9b25-4adc-8b97-576853d16e40", "bd383ff2-258a-41dd-973f-07f7aa30da7a", "3d96ae05-f435-49b3-abd1-574551f38704", "34420358-2236-4010-8504-5fab9af55c0d", "ecc7d466-f9cb-4be1-a099-b8449146cf18", "9d3a7965-d967-42e2-8f25-c0113abaf9ff", "78c650a1-7440-4335-bf95-f421abdf2c0f", "3de7180a-c2f0-48ea-9a9e-6f6e6849c651", "07823fa4-38c9-43fe-8b4b-8aa5588b7c46", "9155e7ae-b4c3-4f06-b8b2-0ec90045ac8e", "e93cee23-c12e-43e4-ac6d-6049aeb13a63", "3cba132d-d6ec-4839-b597-2183f9f998b2", "a84dff3b-2158-41ed-b96c-0b8ace0bea3b", "2632079f-6a54-406a-a957-011c53b16209", "7311aa65-e4bf-4e93-a493-e6bb442ee6a0"]
}

diagram "[PAB] Focus on Network Transport" {
    view: pab
    shows: ["ee7ecd3f-bf35-4657-b7bd-1e45a8fc025c", "22cbea03-a250-4ea8-a6b3-63829196feb1", "6d095cde-706f-41c2-91ab-7c5189f358c4", "809ec42b-c1ee-471f-a45b-d26977dc956a", "b2fe1c12-86f0-4c40-9986-d349c2a09496", "84c8b355-ee9b-48dd-aca8-17b0c4e477c8", "819adf3f-da1d-4d2b-8faf-3b1d7b99503d", "4c361e00-d6d0-4acf-a523-1de11acf7797", "1289a361-a3e8-416b-a9e0-e4fcd528d9e3", "fdc63ce0-6f1d-44f6-942e-8d414a8df2e7", "a850426c-c425-4c85-85fa-cd85b0c066e5", "c1cbb014-c2cd-403b-a9c1-62f438e79c8c", "33ca7fc8-8bd1-48c3-9f22-3fbd217c2549", "aecf56f9-9069-4d1d-87c5-3cd5fbd7e4fe", "c4a29500-aeda-49ed-be51-65ca064266b6", "3f745bea-11ae-4b8c-8a06-930f2391565e", "2dfb0a52-7568-4903-9c78-1e462848f868", "98960be5-3e8f-4743-ab70-805645472512"]
}

diagram "[PDFB] [CTX] Run Cabin Intercommunication Service" {
    view: pab
    shows: ["46a33304-1338-4fc0-a9cb-dc3fe19e5ed4", "9b798c7d-2d5a-43f9-867d-b26c3f4c1337", "cebc5976-81d6-4718-b1f0-9f8b7687ceeb", "1cda3b05-ee17-4912-b875-40bc4ddca86b", "54b31979-1ec1-49c5-81e0-512fd1aec704", "cb0426b2-9cab-4a4d-9e7b-89a13a57be2e", "292159f9-5917-4c0c-9ea3-2800431b3c48"]
}

diagram "[PDFB] Perform remote diagnostic" {
    view: pab
    shows: ["88688938-d911-4317-b36e-266cef11a7f3", "fd6de739-ae93-4e5f-acaf-4e8fb8f0b2e5", "1c49d7b2-bb7a-4939-86c1-a73c0822fdf5", "79218945-69c6-4bc6-8aa6-9d67f537492d", "0012d480-7be3-452f-9872-3a0d8868e2f6", "1291c3c4-170d-4fcc-9b59-5b0961cd4a2d", "71147664-0fcd-401f-937a-bdc76216924d", "62f152bd-6017-4828-9034-7af3c387f83a", "29c4014e-7db7-4660-9f3e-e0a8a37e0090", "52d67cae-5ad2-4c5d-b4bf-e68d308c73b3"]
}

diagram "[PAB] Remote diagnostic equipment SW" {
    view: pab
    shows: ["95ea7294-402d-4783-b6e5-2e405c9f4ccb", "88688938-d911-4317-b36e-266cef11a7f3", "fd6de739-ae93-4e5f-acaf-4e8fb8f0b2e5", "1c49d7b2-bb7a-4939-86c1-a73c0822fdf5", "79218945-69c6-4bc6-8aa6-9d67f537492d", "0012d480-7be3-452f-9872-3a0d8868e2f6", "1291c3c4-170d-4fcc-9b59-5b0961cd4a2d", "71147664-0fcd-401f-937a-bdc76216924d", "62f152bd-6017-4828-9034-7af3c387f83a"]
}

diagram "[SC] System Actors" {
    view: sab
    shows: ["e0a46739-7e50-4ea6-b198-0e5efb9ce68f", "181a678c-dca9-46c1-9d18-b5a0c457c0de", "4890eae8-7a62-4d7b-bf8a-1590a199eebf", "c8b78c78-5b11-4fc0-87b7-3ca84622efea", "d475ed7d-4644-4461-97d4-b52772c9b425"]
}

diagram "[SFBD] All System Functions" {
    view: sab
    shows: ["3398fdbf-3415-4d03-a457-477324d2e091", "cfde8bb1-4df9-4573-a516-41df984ac753", "64847aed-d627-4812-a2de-a5dd8d7ce573", "3688cf59-97ee-4446-8041-7941cee6c767", "880520c2-6bd9-4481-b9da-6ccee7b9d665", "1bc70a69-c6ad-4203-abb9-88b644ac0b95", "65787ee3-ab55-4200-91e3-d772a4ee3f28", "4dbd5747-9662-4378-88cf-270b69ef3827", "e04025f9-6b52-4377-bac1-78d0bf2f8fed", "7de3b04b-6291-4098-afbd-2fb72cd25022", "82ab0ab9-2c03-4689-989a-530aae6fcf94", "6ce8f26a-ef90-4420-94e1-acf6b2393720", "135e77a7-79ef-45e8-b138-67e283a14cdd", "dacbebb6-72c7-4d4c-9009-aa95eba16dd3", "8f279b07-26aa-46f6-8c9a-2a132f0b1177", "6b9e59a9-89f2-497e-a348-3aef80689e61", "9972b7a8-df8a-4626-b298-1dd0d19e451e", "cd62e8fb-8a89-4f33-bd0f-a2bb935a5e08", "4d0046b4-4865-438d-b09b-fd95671db09b", "88faafec-6848-4494-8dd3-2acda3e53688", "ff70c856-f667-4e61-98d7-00190703d53a", "5e0fff47-a1a5-4a13-867c-18348bd894af", "6d10ccfa-1be8-4ab6-a0d8-74941051595b", "6b54b12a-50f8-4ce5-b935-08edbf655d1f", "d683afa5-907e-494d-8f35-e10db839feaf", "a16d1d72-9db2-4554-af57-411b29793904", "8a4b7443-4f73-49c2-a175-940a5d8accea", "b7244d6a-08ab-4455-bbd9-aa5f1bbc9b5b", "2c453928-a098-4f8a-a1e6-fc545a2840d1", "0bfc000c-dd38-40d3-9cd3-2037905a7ab3", "26cd71b0-ec92-41a9-b9fb-6878beede810", "5e221dee-8dc7-4a29-9224-ab53fedb3995", "0a2099f5-2831-4beb-89ae-996a6f54d032", "7b7b0d2c-27e8-4ca0-8a5f-2790a37e97b6", "06b8dd9b-2199-4ad1-a9b2-229d1de1bb48", "6e577fe5-ebd7-409e-b5de-b876fd689851", "00cc4e40-09b1-42c5-b945-32743e1c9968", "f44bd6aa-c54a-49d1-8010-e96dab3ad00a", "4d8cc764-6c72-49ca-8b5e-c57104345aac", "ba4d638d-e6bb-4908-94f4-eb974405c733", "e1808524-dfa9-4883-a809-c44a87712d1b", "d8b8c1bd-a2ea-47fd-a7a9-0239408db1a5", "3f8567f2-5389-42c0-84a9-8a1360dc9572"]
}

diagram "[MCB] All Missions and Capabilities" {
    view: cap
    shows: ["181a678c-dca9-46c1-9d18-b5a0c457c0de", "d475ed7d-4644-4461-97d4-b52772c9b425", "4890eae8-7a62-4d7b-bf8a-1590a199eebf", "c8b78c78-5b11-4fc0-87b7-3ca84622efea", "291fb362-9ae5-4779-8405-1281ea3709f6", "839ba873-8442-4a01-bd1a-1f4c3cc6ed52", "b3a96943-f9e1-4249-97ce-c974ed889f41", "67bacc43-8066-4b96-8101-cf8f3c5f9b35", "43207098-1d88-4680-ae14-68f165e74c24", "4e6fa11e-933f-4aec-b4c0-621c8fb244e5", "4cfe1222-e1e6-4af5-bcb7-b6a51b4ae50c", "9a942752-caaa-4a14-9e18-fca348022a41", "1adec5b0-a145-4a8f-b6c3-35f0fe1ae2d5", "a643f8af-0104-49c8-add7-525806988ccb", "32ff49c3-04bc-476e-b03e-bb6ecbcfe554", "74e6e35e-b1e7-45c1-bd9f-5d80c0714472", "5acd248a-3961-4e71-85a4-5bc3b775215f", "67e5b193-060d-43ff-b9bb-7bd14bcf425a", "eee7180d-9cb1-4253-9ecb-2e5c811db2c9", "3d3f045d-4c6e-4b2c-8550-5e1ddd6ce87a", "e9691a85-30a6-4b72-842d-b0e59ca2a104", "cdfe2a5a-5069-4055-be2c-96be11552fa9"]
}

diagram "[SAB] High Level System Overview" {
    view: sab
    shows: ["181a678c-dca9-46c1-9d18-b5a0c457c0de", "880520c2-6bd9-4481-b9da-6ccee7b9d665", "65787ee3-ab55-4200-91e3-d772a4ee3f28", "4dbd5747-9662-4378-88cf-270b69ef3827", "1bc70a69-c6ad-4203-abb9-88b644ac0b95", "e04025f9-6b52-4377-bac1-78d0bf2f8fed", "d475ed7d-4644-4461-97d4-b52772c9b425", "4d0046b4-4865-438d-b09b-fd95671db09b", "5e0fff47-a1a5-4a13-867c-18348bd894af", "88faafec-6848-4494-8dd3-2acda3e53688", "ff70c856-f667-4e61-98d7-00190703d53a", "4890eae8-7a62-4d7b-bf8a-1590a199eebf", "cfde8bb1-4df9-4573-a516-41df984ac753", "64847aed-d627-4812-a2de-a5dd8d7ce573", "c8b78c78-5b11-4fc0-87b7-3ca84622efea", "6b9e59a9-89f2-497e-a348-3aef80689e61", "82ab0ab9-2c03-4689-989a-530aae6fcf94", "6ce8f26a-ef90-4420-94e1-acf6b2393720", "8f279b07-26aa-46f6-8c9a-2a132f0b1177", "dacbebb6-72c7-4d4c-9009-aa95eba16dd3", "135e77a7-79ef-45e8-b138-67e283a14cdd", "9972b7a8-df8a-4626-b298-1dd0d19e451e", "e0a46739-7e50-4ea6-b198-0e5efb9ce68f", "8a4b7443-4f73-49c2-a175-940a5d8accea", "d683afa5-907e-494d-8f35-e10db839feaf", "a16d1d72-9db2-4554-af57-411b29793904", "7b7b0d2c-27e8-4ca0-8a5f-2790a37e97b6", "26cd71b0-ec92-41a9-b9fb-6878beede810", "ba4d638d-e6bb-4908-94f4-eb974405c733", "3f8567f2-5389-42c0-84a9-8a1360dc9572", "00cc4e40-09b1-42c5-b945-32743e1c9968", "f44bd6aa-c54a-49d1-8010-e96dab3ad00a", "4d8cc764-6c72-49ca-8b5e-c57104345aac"]
}

diagram "[SDFB] [CTX] Manage Passenger Services Lifecycle" {
    view: sab
    shows: ["8a4b7443-4f73-49c2-a175-940a5d8accea", "b7244d6a-08ab-4455-bbd9-aa5f1bbc9b5b", "2c453928-a098-4f8a-a1e6-fc545a2840d1", "0bfc000c-dd38-40d3-9cd3-2037905a7ab3", "5e0fff47-a1a5-4a13-867c-18348bd894af", "65787ee3-ab55-4200-91e3-d772a4ee3f28", "3f8567f2-5389-42c0-84a9-8a1360dc9572"]
}

diagram "[SDFB] [BUILD] All Functions" {
    view: sab
    shows: ["7de3b04b-6291-4098-afbd-2fb72cd25022", "8f279b07-26aa-46f6-8c9a-2a132f0b1177", "82ab0ab9-2c03-4689-989a-530aae6fcf94", "9972b7a8-df8a-4626-b298-1dd0d19e451e", "6b9e59a9-89f2-497e-a348-3aef80689e61", "dacbebb6-72c7-4d4c-9009-aa95eba16dd3", "6ce8f26a-ef90-4420-94e1-acf6b2393720", "135e77a7-79ef-45e8-b138-67e283a14cdd", "3398fdbf-3415-4d03-a457-477324d2e091", "cfde8bb1-4df9-4573-a516-41df984ac753", "64847aed-d627-4812-a2de-a5dd8d7ce573", "3688cf59-97ee-4446-8041-7941cee6c767", "65787ee3-ab55-4200-91e3-d772a4ee3f28", "4dbd5747-9662-4378-88cf-270b69ef3827", "880520c2-6bd9-4481-b9da-6ccee7b9d665", "1bc70a69-c6ad-4203-abb9-88b644ac0b95", "e04025f9-6b52-4377-bac1-78d0bf2f8fed", "cd62e8fb-8a89-4f33-bd0f-a2bb935a5e08", "5e0fff47-a1a5-4a13-867c-18348bd894af", "4d0046b4-4865-438d-b09b-fd95671db09b", "88faafec-6848-4494-8dd3-2acda3e53688", "ff70c856-f667-4e61-98d7-00190703d53a", "6d10ccfa-1be8-4ab6-a0d8-74941051595b", "ba4d638d-e6bb-4908-94f4-eb974405c733", "e1808524-dfa9-4883-a809-c44a87712d1b", "d8b8c1bd-a2ea-47fd-a7a9-0239408db1a5", "3f8567f2-5389-42c0-84a9-8a1360dc9572", "6b54b12a-50f8-4ce5-b935-08edbf655d1f", "a16d1d72-9db2-4554-af57-411b29793904", "d683afa5-907e-494d-8f35-e10db839feaf", "0bfc000c-dd38-40d3-9cd3-2037905a7ab3", "26cd71b0-ec92-41a9-b9fb-6878beede810", "0a2099f5-2831-4beb-89ae-996a6f54d032", "5e221dee-8dc7-4a29-9224-ab53fedb3995", "7b7b0d2c-27e8-4ca0-8a5f-2790a37e97b6", "6e577fe5-ebd7-409e-b5de-b876fd689851", "06b8dd9b-2199-4ad1-a9b2-229d1de1bb48", "00cc4e40-09b1-42c5-b945-32743e1c9968", "f44bd6aa-c54a-49d1-8010-e96dab3ad00a", "4d8cc764-6c72-49ca-8b5e-c57104345aac", "8a4b7443-4f73-49c2-a175-940a5d8accea", "b7244d6a-08ab-4455-bbd9-aa5f1bbc9b5b", "2c453928-a098-4f8a-a1e6-fc545a2840d1"]
}

diagram "[SFBD] All Actor Functions" {
    view: sab
    shows: ["3398fdbf-3415-4d03-a457-477324d2e091", "cfde8bb1-4df9-4573-a516-41df984ac753", "64847aed-d627-4812-a2de-a5dd8d7ce573", "3688cf59-97ee-4446-8041-7941cee6c767", "880520c2-6bd9-4481-b9da-6ccee7b9d665", "1bc70a69-c6ad-4203-abb9-88b644ac0b95", "65787ee3-ab55-4200-91e3-d772a4ee3f28", "4dbd5747-9662-4378-88cf-270b69ef3827", "e04025f9-6b52-4377-bac1-78d0bf2f8fed", "7de3b04b-6291-4098-afbd-2fb72cd25022", "82ab0ab9-2c03-4689-989a-530aae6fcf94", "6ce8f26a-ef90-4420-94e1-acf6b2393720", "135e77a7-79ef-45e8-b138-67e283a14cdd", "dacbebb6-72c7-4d4c-9009-aa95eba16dd3", "8f279b07-26aa-46f6-8c9a-2a132f0b1177", "6b9e59a9-89f2-497e-a348-3aef80689e61", "9972b7a8-df8a-4626-b298-1dd0d19e451e", "cd62e8fb-8a89-4f33-bd0f-a2bb935a5e08", "4d0046b4-4865-438d-b09b-fd95671db09b", "88faafec-6848-4494-8dd3-2acda3e53688", "ff70c856-f667-4e61-98d7-00190703d53a", "5e0fff47-a1a5-4a13-867c-18348bd894af", "6d10ccfa-1be8-4ab6-a0d8-74941051595b", "6b54b12a-50f8-4ce5-b935-08edbf655d1f", "d683afa5-907e-494d-8f35-e10db839feaf", "a16d1d72-9db2-4554-af57-411b29793904", "8a4b7443-4f73-49c2-a175-940a5d8accea", "b7244d6a-08ab-4455-bbd9-aa5f1bbc9b5b", "2c453928-a098-4f8a-a1e6-fc545a2840d1", "0bfc000c-dd38-40d3-9cd3-2037905a7ab3", "26cd71b0-ec92-41a9-b9fb-6878beede810", "5e221dee-8dc7-4a29-9224-ab53fedb3995", "0a2099f5-2831-4beb-89ae-996a6f54d032", "7b7b0d2c-27e8-4ca0-8a5f-2790a37e97b6", "06b8dd9b-2199-4ad1-a9b2-229d1de1bb48", "6e577fe5-ebd7-409e-b5de-b876fd689851", "00cc4e40-09b1-42c5-b945-32743e1c9968", "f44bd6aa-c54a-49d1-8010-e96dab3ad00a", "4d8cc764-6c72-49ca-8b5e-c57104345aac", "ba4d638d-e6bb-4908-94f4-eb974405c733", "e1808524-dfa9-4883-a809-c44a87712d1b", "d8b8c1bd-a2ea-47fd-a7a9-0239408db1a5", "3f8567f2-5389-42c0-84a9-8a1360dc9572"]
}

diagram "[SDFB] [CTX] Run Cabin Intercommunication Service" {
    view: sab
    shows: ["7b7b0d2c-27e8-4ca0-8a5f-2790a37e97b6", "06b8dd9b-2199-4ad1-a9b2-229d1de1bb48", "6e577fe5-ebd7-409e-b5de-b876fd689851", "cfde8bb1-4df9-4573-a516-41df984ac753", "d683afa5-907e-494d-8f35-e10db839feaf", "ba4d638d-e6bb-4908-94f4-eb974405c733", "8a4b7443-4f73-49c2-a175-940a5d8accea"]
}

diagram "[SDFB] [CTX] Run Video-On-Demand Service" {
    view: sab
    shows: ["26cd71b0-ec92-41a9-b9fb-6878beede810", "5e221dee-8dc7-4a29-9224-ab53fedb3995", "0a2099f5-2831-4beb-89ae-996a6f54d032", "d683afa5-907e-494d-8f35-e10db839feaf", "4dbd5747-9662-4378-88cf-270b69ef3827", "ba4d638d-e6bb-4908-94f4-eb974405c733", "8a4b7443-4f73-49c2-a175-940a5d8accea"]
}

diagram "[SDFB] [CTX] Manage Video and Audio Diffusion" {
    view: sab
    shows: ["ba4d638d-e6bb-4908-94f4-eb974405c733", "d8b8c1bd-a2ea-47fd-a7a9-0239408db1a5", "e1808524-dfa9-4883-a809-c44a87712d1b", "0bfc000c-dd38-40d3-9cd3-2037905a7ab3", "d683afa5-907e-494d-8f35-e10db839feaf", "cfde8bb1-4df9-4573-a516-41df984ac753", "880520c2-6bd9-4481-b9da-6ccee7b9d665", "1bc70a69-c6ad-4203-abb9-88b644ac0b95", "e04025f9-6b52-4377-bac1-78d0bf2f8fed", "6b9e59a9-89f2-497e-a348-3aef80689e61", "8f279b07-26aa-46f6-8c9a-2a132f0b1177"]
}

diagram "[SAB] Top Level System Overview" {
    view: sab
    shows: ["181a678c-dca9-46c1-9d18-b5a0c457c0de", "3688cf59-97ee-4446-8041-7941cee6c767", "d475ed7d-4644-4461-97d4-b52772c9b425", "cd62e8fb-8a89-4f33-bd0f-a2bb935a5e08", "4890eae8-7a62-4d7b-bf8a-1590a199eebf", "3398fdbf-3415-4d03-a457-477324d2e091", "c8b78c78-5b11-4fc0-87b7-3ca84622efea", "7de3b04b-6291-4098-afbd-2fb72cd25022", "e0a46739-7e50-4ea6-b198-0e5efb9ce68f", "8a4b7443-4f73-49c2-a175-940a5d8accea", "ba4d638d-e6bb-4908-94f4-eb974405c733", "0bfc000c-dd38-40d3-9cd3-2037905a7ab3", "6b54b12a-50f8-4ce5-b935-08edbf655d1f", "3f8567f2-5389-42c0-84a9-8a1360dc9572"]
}

diagram "[SDFB] Top Level Functional Overview" {
    view: sab
    shows: ["6d10ccfa-1be8-4ab6-a0d8-74941051595b", "3398fdbf-3415-4d03-a457-477324d2e091", "3688cf59-97ee-4446-8041-7941cee6c767", "cd62e8fb-8a89-4f33-bd0f-a2bb935a5e08", "7de3b04b-6291-4098-afbd-2fb72cd25022"]
}

diagram "[SDFB] [CAPABILITY] Provide Audio and Video Intercommunication Means" {
    view: sab
    shows: ["6b9e59a9-89f2-497e-a348-3aef80689e61", "d8b8c1bd-a2ea-47fd-a7a9-0239408db1a5", "1bc70a69-c6ad-4203-abb9-88b644ac0b95", "d683afa5-907e-494d-8f35-e10db839feaf", "06b8dd9b-2199-4ad1-a9b2-229d1de1bb48", "cfde8bb1-4df9-4573-a516-41df984ac753", "e1808524-dfa9-4883-a809-c44a87712d1b", "880520c2-6bd9-4481-b9da-6ccee7b9d665", "e04025f9-6b52-4377-bac1-78d0bf2f8fed"]
}

diagram "[SDFB] [CTX] Run Services" {
    view: sab
    shows: ["0bfc000c-dd38-40d3-9cd3-2037905a7ab3", "f44bd6aa-c54a-49d1-8010-e96dab3ad00a", "4d8cc764-6c72-49ca-8b5e-c57104345aac", "26cd71b0-ec92-41a9-b9fb-6878beede810", "00cc4e40-09b1-42c5-b945-32743e1c9968", "7b7b0d2c-27e8-4ca0-8a5f-2790a37e97b6", "3398fdbf-3415-4d03-a457-477324d2e091", "3688cf59-97ee-4446-8041-7941cee6c767", "8a4b7443-4f73-49c2-a175-940a5d8accea", "6b54b12a-50f8-4ce5-b935-08edbf655d1f", "ba4d638d-e6bb-4908-94f4-eb974405c733"]
}

diagram "[SDFB] [CAPABILITY] Provide Video Entertainment Services" {
    view: sab
    shows: ["880520c2-6bd9-4481-b9da-6ccee7b9d665", "d683afa5-907e-494d-8f35-e10db839feaf", "4dbd5747-9662-4378-88cf-270b69ef3827", "e1808524-dfa9-4883-a809-c44a87712d1b", "d8b8c1bd-a2ea-47fd-a7a9-0239408db1a5", "5e221dee-8dc7-4a29-9224-ab53fedb3995", "0a2099f5-2831-4beb-89ae-996a6f54d032", "b7244d6a-08ab-4455-bbd9-aa5f1bbc9b5b", "65787ee3-ab55-4200-91e3-d772a4ee3f28", "2c453928-a098-4f8a-a1e6-fc545a2840d1"]
}

diagram "[CM] Provide Entertainment Solutions" {
    view: cap
    shows: ["c8b78c78-5b11-4fc0-87b7-3ca84622efea", "181a678c-dca9-46c1-9d18-b5a0c457c0de", "4e6fa11e-933f-4aec-b4c0-621c8fb244e5", "4cfe1222-e1e6-4af5-bcb7-b6a51b4ae50c", "839ba873-8442-4a01-bd1a-1f4c3cc6ed52", "b3a96943-f9e1-4249-97ce-c974ed889f41", "67e5b193-060d-43ff-b9bb-7bd14bcf425a"]
}

diagram "[CM] Provide Cabin Management Solutions" {
    view: cap
    shows: ["4890eae8-7a62-4d7b-bf8a-1590a199eebf", "181a678c-dca9-46c1-9d18-b5a0c457c0de", "c8b78c78-5b11-4fc0-87b7-3ca84622efea", "291fb362-9ae5-4779-8405-1281ea3709f6", "eee7180d-9cb1-4253-9ecb-2e5c811db2c9"]
}

diagram "[CC] Provide Audio and Video Intercommunication Means" {
    view: cap
    shows: ["4890eae8-7a62-4d7b-bf8a-1590a199eebf", "181a678c-dca9-46c1-9d18-b5a0c457c0de", "c8b78c78-5b11-4fc0-87b7-3ca84622efea", "291fb362-9ae5-4779-8405-1281ea3709f6", "eee7180d-9cb1-4253-9ecb-2e5c811db2c9"]
}

diagram "[CC] Provide Video Entertainment Services" {
    view: cap
    shows: ["181a678c-dca9-46c1-9d18-b5a0c457c0de", "839ba873-8442-4a01-bd1a-1f4c3cc6ed52", "67e5b193-060d-43ff-b9bb-7bd14bcf425a"]
}

diagram "[SAB] High Level System Overview with some FCs" {
    view: sab
    shows: ["181a678c-dca9-46c1-9d18-b5a0c457c0de", "880520c2-6bd9-4481-b9da-6ccee7b9d665", "65787ee3-ab55-4200-91e3-d772a4ee3f28", "4dbd5747-9662-4378-88cf-270b69ef3827", "1bc70a69-c6ad-4203-abb9-88b644ac0b95", "e04025f9-6b52-4377-bac1-78d0bf2f8fed", "d475ed7d-4644-4461-97d4-b52772c9b425", "4d0046b4-4865-438d-b09b-fd95671db09b", "5e0fff47-a1a5-4a13-867c-18348bd894af", "88faafec-6848-4494-8dd3-2acda3e53688", "ff70c856-f667-4e61-98d7-00190703d53a", "4890eae8-7a62-4d7b-bf8a-1590a199eebf", "cfde8bb1-4df9-4573-a516-41df984ac753", "64847aed-d627-4812-a2de-a5dd8d7ce573", "c8b78c78-5b11-4fc0-87b7-3ca84622efea", "6b9e59a9-89f2-497e-a348-3aef80689e61", "82ab0ab9-2c03-4689-989a-530aae6fcf94", "6ce8f26a-ef90-4420-94e1-acf6b2393720", "8f279b07-26aa-46f6-8c9a-2a132f0b1177", "dacbebb6-72c7-4d4c-9009-aa95eba16dd3", "135e77a7-79ef-45e8-b138-67e283a14cdd", "9972b7a8-df8a-4626-b298-1dd0d19e451e", "e0a46739-7e50-4ea6-b198-0e5efb9ce68f", "8a4b7443-4f73-49c2-a175-940a5d8accea", "d683afa5-907e-494d-8f35-e10db839feaf", "a16d1d72-9db2-4554-af57-411b29793904", "7b7b0d2c-27e8-4ca0-8a5f-2790a37e97b6", "26cd71b0-ec92-41a9-b9fb-6878beede810", "ba4d638d-e6bb-4908-94f4-eb974405c733", "3f8567f2-5389-42c0-84a9-8a1360dc9572", "00cc4e40-09b1-42c5-b945-32743e1c9968", "f44bd6aa-c54a-49d1-8010-e96dab3ad00a", "4d8cc764-6c72-49ca-8b5e-c57104345aac", "dd56a4c7-8304-4471-bf3f-9ffddaccf544", "adb3033a-d058-4995-98e9-60aaf85f4328", "f5c2a034-4749-41c2-a77e-6ecaae28f5d0"]
}

