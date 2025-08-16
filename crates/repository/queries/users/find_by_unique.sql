SELECT
  u.id,
  u.email,
  u.verified,
  u.display_name,
  u.image_url AS "image_url: DbUrl",
  coalesce(
    jsonb_agg(
      jsonb_build_object(
        'code',
        oc.code,
        'expires_at',
        oc.expires_at,
        'used_at',
        oc.used_at,
        'created_at',
        oc.created_at,
        'updated_at',
        oc.updated_at
      )
      ORDER BY
        oc.created_at ASC
    ) FILTER (
      WHERE
        oc.code IS NOT NULL
    ),
    '[]'::JSONB
  ) AS "otp_codes!: Json<Vec<OtpCodeRow>>",
  coalesce(
    jsonb_agg(
      jsonb_build_object(
        'provider',
        sa.provider,
        'sub',
        sa.sub,
        'created_at',
        sa.created_at,
        'updated_at',
        sa.updated_at
      )
      ORDER BY
        sa.provider ASC,
        sa.sub ASC
    ) FILTER (
      WHERE
        sa.sub IS NOT NULL
    ),
    '[]'::JSONB
  ) AS "social_accounts!: Json<Vec<SocialAccountRow>>",
  coalesce(
    jsonb_agg(
      jsonb_build_object(
        'id',
        pat.id,
        'lookup_hash',
        pat.lookup_hash,
        'verification_hash',
        pat.verification_hash,
        'title',
        pat.title,
        'preview',
        pat.preview,
        'created_at',
        pat.created_at,
        'updated_at',
        pat.updated_at
      )
      ORDER BY
        pat.created_at ASC
    ) FILTER (
      WHERE
        pat.id IS NOT NULL
    ),
    '[]'::JSONB
  ) AS "personal_access_tokens!: Json<Vec<PersonalAccessTokenRow>>",
  u.created_at,
  u.updated_at
FROM
  users u
  LEFT JOIN otp_codes oc ON oc.user_id = u.id
  LEFT JOIN social_accounts sa ON sa.user_id = u.id
  LEFT JOIN personal_access_tokens pat ON pat.user_id = u.id
WHERE
  (
    $1::UUID IS NULL
    OR u.id = $1
  )
  AND (
    $2::TEXT IS NULL
    OR u.email = $2
  )
  AND (
    (
      $3::TEXT IS NULL
      AND $4::TEXT IS NULL
    )
    OR u.id IN (
      SELECT
        user_id
      FROM
        social_accounts
      WHERE
        provider = $3
        AND sub = $4
    )
  )
GROUP BY
  u.id
