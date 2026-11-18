// A link shortener: a system made of software only. No sensor, no ECU, no
// safety level: services, a database, a cache and a queue, with the
// requirements a product review asks of them (latency, availability, abuse)
// followed through the four Arcadia levels down to the containers they run in.
//
//   arclang check examples/software/link-shortener/link_shortener.arc
//   arclang gate  examples/software/link-shortener/link_shortener.arc

model LinkShortener {
    name: "Link shortener (a system made of software only)"
    version: "1.0"
}

requirements {
    requirement "REQ-LNK-001" {
        description: "A short link shall redirect its visitor to the long address in less than 50 ms at the server"
        priority: "Critical"
    }
    requirement "REQ-LNK-002" {
        description: "A short link, once given, shall keep answering: redirects shall be available 99.95 % of the month"
        priority: "Critical"
    }
    requirement "REQ-LNK-003" {
        description: "A publisher shall see how many visits each of its links had, at most one minute late"
        priority: "Medium"
    }
    requirement "REQ-LNK-004" {
        description: "A link reported as abuse shall stop redirecting within five minutes of its confirmation"
        priority: "High"
    }
    requirement "REQ-LNK-005" {
        description: "Counting the visits shall never slow a redirect down"
        priority: "High"
    }
}

operational_analysis "Sharing links" {
    actor "Publisher" { id: "OA-PUB" description: "Shortens an address to share it, and follows how it is used" }
    actor "Visitor" { id: "OA-VIS" description: "Follows a short link, without an account" }
    entity "Link service" {
        id: "OA-SVC"
        activity "Give a short address" { id: "OA-GIVE" }
        activity "Send a visitor to the long address" { id: "OA-SEND" }
        activity "Report how the links are used" { id: "OA-REPORT" }
    }
    entity "Moderation team" {
        id: "OA-MOD"
        activity "Withdraw an abusive link" { id: "OA-WITHDRAW" }
    }
    operational_capability "Share a long address as a short one" {
        id: "OC-SHARE"
        involves: ["OA-PUB", "OA-VIS", "OA-SVC"]
    }
    operational_capability "Keep abuse out" {
        id: "OC-ABUSE"
        involves: ["OA-MOD", "OA-SVC"]
    }
}

system_analysis "Link shortener" {
    actor "Publisher" { id: "SA-PUB" }
    actor "Visitor" { id: "SA-VIS" }
    actor "Moderator" { id: "SA-MOD" }
    capability "Shorten and redirect" { id: "CAP-SHARE" realizes: "OC-SHARE" involves: ["SF-SHORTEN", "SF-REDIRECT", "SF-COUNT"] }
    capability "Withdraw a link" { id: "CAP-ABUSE" realizes: "OC-ABUSE" involves: ["SF-WITHDRAW", "SF-REDIRECT"] }
    function "Shorten an address" { id: "SF-SHORTEN" latency: 80 ms port in "long address" port out "link" }
    function "Redirect a visitor" { id: "SF-REDIRECT" latency: 20 ms port in "link" port in "withdrawal" port out "visit" }
    function "Count the visits" { id: "SF-COUNT" latency: 30 s port in "visit" port out "statistics" }
    function "Withdraw a link" { id: "SF-WITHDRAW" latency: 2 s port in "report" port out "withdrawal" }
    functional_exchange "new link" { from: "SF-SHORTEN.link" to: "SF-REDIRECT.link" }
    functional_exchange "visit" { from: "SF-REDIRECT.visit" to: "SF-COUNT.visit" }
    functional_exchange "withdrawal" { from: "SF-WITHDRAW.withdrawal" to: "SF-REDIRECT.withdrawal" }
    functional_chain "A visit, from the click to the count" { id: "FC-VISIT" involves: ["SF-REDIRECT", "SF-COUNT"] latency_budget: 60 s }
}

logical_architecture "Services" {
    component "Public API" {
        id: "LC-API"
        description: "The only door for publishers and moderators: authenticates, validates, rate-limits"
        port in "requests"
        port out "links"
        port out "withdrawals"
        function "Create a link" { id: "LF-CREATE" realizes: "SF-SHORTEN" }
        function "Take a confirmed report" { id: "LF-REPORT" realizes: "SF-WITHDRAW" }
    }
    component "Redirector" {
        id: "LC-RED"
        description: "Answers a short address from memory; never waits for the statistics"
        port in "links"
        port in "withdrawals"
        port out "visits"
        function "Resolve and redirect" { id: "LF-RESOLVE" realizes: "SF-REDIRECT" }
    }
    component "Link store" {
        id: "LC-STORE"
        description: "The links, their owners and their state: the record that must not be lost"
        port in "links"
        port out "known links"
        function "Keep the links" { id: "LF-KEEP" }
    }
    component "Statistics" {
        id: "LC-STAT"
        description: "Counts the visits after the fact, from the queue"
        port in "visits"
        port out "counts"
        function "Aggregate the visits" { id: "LF-AGG" realizes: "SF-COUNT" }
    }
    component_exchange "Created links" { from_port: "LC-API.links" to_port: "LC-STORE.links" }
    component_exchange "Links to answer" { from_port: "LC-STORE.known links" to_port: "LC-RED.links" }
    component_exchange "Withdrawals" { from_port: "LC-API.withdrawals" to_port: "LC-RED.withdrawals" }
    component_exchange "Visit events" { from_port: "LC-RED.visits" to_port: "LC-STAT.visits" }
}

physical_architecture "Deployment" {
    node "Edge" { id: "PN-EDGE" description: "Three regions, the redirector beside a cache of the links" replicas: 6 deploys "LC-RED" }
    node "Application cluster" { id: "PN-APP" description: "The API and the statistics workers, as containers" replicas: 3 deploys "LC-API" deploys "LC-STAT" }
    node "Database" { id: "PN-DB" description: "PostgreSQL, one primary and two replicas" deploys "LC-STORE" }
    physical_link "Link replication" { id: "PL-REPL" from: "PN-DB" to: "PN-EDGE" protocol: "logical replication" }
    physical_link "Visit queue" { id: "PL-QUEUE" from: "PN-EDGE" to: "PN-APP" protocol: "Kafka" }
    physical_link "SQL" { id: "PL-SQL" from: "PN-APP" to: "PN-DB" protocol: "PostgreSQL wire protocol" }
}

test_case "TC-LNK-001" { verifies: ["REQ-LNK-001"] method: "test" description: "Load test: 20 000 redirects a second, 99th percentile at the server" verdict: "pass" }
test_case "TC-LNK-002" { verifies: ["REQ-LNK-002"] method: "analysis" description: "Availability computed from the redundancy of the edge and of the database" verdict: "pass" }
test_case "TC-LNK-003" { verifies: ["REQ-LNK-003"] method: "test" description: "A visit shows in the publisher's page within a minute" verdict: "pass" }
test_case "TC-LNK-004" { verifies: ["REQ-LNK-004"] method: "test" description: "A withdrawn link answers 410 in every region within five minutes" verdict: "pass" }
test_case "TC-LNK-005" { verifies: ["REQ-LNK-005"] method: "test" description: "Redirect latency unchanged with the statistics workers stopped" verdict: "pass" }

trace "LC-RED" satisfies "REQ-LNK-001" { rationale: "The redirector answers from memory, beside the visitor" }
trace "PN-EDGE" satisfies "REQ-LNK-002" { rationale: "Six replicas over three regions keep redirecting when one region is lost" }
trace "LC-STORE" satisfies "REQ-LNK-002" { rationale: "The links are kept on a primary and two replicas: losing one loses no link" }
trace "LC-STAT" satisfies "REQ-LNK-003" { rationale: "The visits are aggregated every thirty seconds" }
trace "LC-API" satisfies "REQ-LNK-004" { rationale: "A confirmed report is pushed to the redirectors at once" }
trace "LC-RED" satisfies "REQ-LNK-004" { rationale: "A withdrawn link is refused from memory" }
trace "LC-RED" satisfies "REQ-LNK-005" { rationale: "A visit is an event put on the queue, never a call that is waited for" }
trace "SF-REDIRECT" realizes "OA-SEND"
trace "SF-SHORTEN" realizes "OA-GIVE"
trace "SF-COUNT" realizes "OA-REPORT"
trace "SF-WITHDRAW" realizes "OA-WITHDRAW"
