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
  u.created_at,
  u.updated_at
FROM
  users u
  LEFT JOIN otp_codes oc ON oc.user_id = u.id
  LEFT JOIN social_accounts sa ON sa.user_id = u.id
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
