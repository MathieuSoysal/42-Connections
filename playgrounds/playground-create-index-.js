use('42');
db.getCollection('profiles').aggregate([
  {
    $project: {
      _id: 1,
    },
  },
  {
    $addFields: {
      page_number: 1,
    },
  },
  {
    $merge: {
      into: { db: "application", coll: "events_participation_index" },
      whenMatched: "merge",
      whenNotMatched: "insert",
    },
  },
]);