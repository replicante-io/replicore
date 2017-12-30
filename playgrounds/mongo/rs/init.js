let status = rs.status();
if (status.codeName == "NotYetInitialized") {
    print("Replica Set not initialised, correcting ...");
    rs.initiate({
        _id: "replica-set",
        members: [
            {_id: 0, host: "node1:27017"},
            {_id: 1, host: "node2:27017"},
            {_id: 2, host: "node3:27017"}
        ]
    });
    print("Replica Set Ready!");
} else {
    print("Replica Set initialised, nothing to do");
}
