use('42');
db.event_participations.aggregate([
    { $unwind: "$events" },
    { $group: { _id: "$events.event_id" } },
    {
      $merge: {
        into: { db: "application", coll: "events_ids" },
        whenMatched: "keepExisting",
        whenNotMatched: "insert"
      }
    }
  ]);