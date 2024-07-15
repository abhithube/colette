-- Custom SQL migration file, put you code below! --

CREATE OR REPLACE FUNCTION handle_updated_at ()
  RETURNS TRIGGER
  AS $$
BEGIN
  NEW.updated_at = now();
  RETURN NEW;
END;
$$
LANGUAGE 'plpgsql';

CREATE TRIGGER users_updated_at
  BEFORE UPDATE ON users FOR EACH ROW
  EXECUTE PROCEDURE handle_updated_at ();

CREATE TRIGGER profiles_updated_at
  BEFORE UPDATE ON profiles FOR EACH ROW
  EXECUTE PROCEDURE handle_updated_at ();
