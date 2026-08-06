DO $$
BEGIN
    RAISE EXCEPTION 'This production migration is irreversible. Restore from a verified backup instead of rolling it back.';
END
$$;

