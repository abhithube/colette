CREATE FUNCTION set_updated_at () returns trigger AS $$ begin
  NEW.updated_at := now();
  return NEW;
end; $$ language plpgsql;

CREATE TRIGGER set_updated_at_users before
UPDATE ON users FOR each ROW
EXECUTE procedure set_updated_at ();

CREATE TRIGGER set_updated_at_feeds before
UPDATE ON feeds FOR each ROW
EXECUTE procedure set_updated_at ();

CREATE TRIGGER set_updated_at_feed_entries before
UPDATE ON feed_entries FOR each ROW
EXECUTE procedure set_updated_at ();

CREATE TRIGGER set_updated_at_folders before
UPDATE ON folders FOR each ROW
EXECUTE procedure set_updated_at ();

CREATE TRIGGER set_updated_at_user_feeds before
UPDATE ON user_feeds FOR each ROW
EXECUTE procedure set_updated_at ();

CREATE TRIGGER set_updated_at_user_feed_entries before
UPDATE ON user_feed_entries FOR each ROW
EXECUTE procedure set_updated_at ();

CREATE TRIGGER set_updated_at_bookmarks before
UPDATE ON bookmarks FOR each ROW
EXECUTE procedure set_updated_at ();

CREATE TRIGGER set_updated_at_tags before
UPDATE ON tags FOR each ROW
EXECUTE procedure set_updated_at ();

CREATE TRIGGER set_updated_at_user_feed_tags before
UPDATE ON user_feed_tags FOR each ROW
EXECUTE procedure set_updated_at ();

CREATE TRIGGER set_updated_at_bookmark_tags before
UPDATE ON bookmark_tags FOR each ROW
EXECUTE procedure set_updated_at ();
