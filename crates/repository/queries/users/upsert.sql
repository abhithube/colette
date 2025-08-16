WITH
  input_oc AS (
    SELECT
      *,
      $1::UUID AS user_id
    FROM
      unnest(
        $8::TEXT[],
        $9::TIMESTAMPTZ[],
        $10::TIMESTAMPTZ[],
        $11::TIMESTAMPTZ[],
        $12::TIMESTAMPTZ[]
      ) AS oc (code, expires_at, used_at, created_at, updated_at)
  ),
  input_sa AS (
    SELECT
      *,
      $1::UUID AS user_id
    FROM
      unnest(
        $13::TEXT[],
        $14::TEXT[],
        $15::TIMESTAMPTZ[],
        $16::TIMESTAMPTZ[]
      ) AS sa (provider, sub, created_at, updated_at)
  ),
  input_pat AS (
    SELECT
      *,
      $1::UUID AS user_id
    FROM
      unnest(
        $17::UUID[],
        $18::TEXT[],
        $19::TEXT[],
        $20::TEXT[],
        $21::TEXT[],
        $22::TIMESTAMPTZ[],
        $23::TIMESTAMPTZ[]
      ) AS pat (
        id,
        lookup_hash,
        verification_hash,
        title,
        preview,
        created_at,
        updated_at
      )
  ),
  upserted_user AS (
    INSERT INTO
      users (
        id,
        email,
        verified,
        display_name,
        image_url,
        created_at,
        updated_at
      )
    VALUES
      ($1, $2, $3, $4, $5, $6, $7)
    ON CONFLICT (id) DO UPDATE
    SET
      email = EXCLUDED.email,
      verified = EXCLUDED.verified,
      display_name = EXCLUDED.display_name,
      image_url = EXCLUDED.image_url,
      updated_at = EXCLUDED.updated_at
    RETURNING
      id
  ),
  inserted_sa AS (
    INSERT INTO
      social_accounts (provider, sub, user_id, created_at, updated_at)
    SELECT
      provider,
      sub,
      user_id,
      created_at,
      updated_at
    FROM
      input_sa
    ON CONFLICT (provider, sub) DO NOTHING
  ),
  inserted_oc AS (
    INSERT INTO
      otp_codes (
        code,
        expires_at,
        used_at,
        user_id,
        created_at,
        updated_at
      )
    SELECT
      code,
      expires_at,
      used_at,
      user_id,
      created_at,
      updated_at
    FROM
      input_oc
    ON CONFLICT (user_id, code) DO UPDATE
    SET
      expires_at = EXCLUDED.expires_at,
      used_at = EXCLUDED.used_at,
      updated_at = EXCLUDED.updated_at
  ),
  deleted_sa AS (
    DELETE FROM social_accounts old USING input_sa sa
    WHERE
      NOT old.provider = sa.provider
      AND NOT old.sub = sa.sub
  ),
  inserted_pat AS (
    INSERT INTO
      personal_access_tokens (
        id,
        lookup_hash,
        verification_hash,
        title,
        preview,
        user_id,
        created_at,
        updated_at
      )
    SELECT
      id,
      lookup_hash,
      verification_hash,
      title,
      preview,
      user_id,
      created_at,
      updated_at
    FROM
      input_pat
    ON CONFLICT (id) DO UPDATE
    SET
      title = EXCLUDED.title,
      updated_at = EXCLUDED.updated_at
  )
DELETE FROM personal_access_tokens old USING input_pat pat
WHERE
  old.user_id = $1
  AND NOT old.id = pat.id
