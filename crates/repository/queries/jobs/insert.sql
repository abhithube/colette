INSERT INTO
  jobs (job_type, data_json, group_identifier)
VALUES
  ($1, $2, $3)
RETURNING
  id
