SELECT
  id,
  status AS "status: DbJobStatus"
FROM
  jobs
WHERE
  id = $1
