SELECT
  id,
  job_type,
  data_json AS "data_json: Json<Value>",
  status AS "status: DbJobStatus",
  group_identifier,
  message,
  created_at,
  updated_at
FROM
  jobs
WHERE
  (
    $1::UUID IS NULL
    OR id = $1
  )
  AND (
    $2::TEXT IS NULL
    OR group_identifier = $2
  )
ORDER BY
  created_at ASC
