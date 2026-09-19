def patch(entries, patches, data):
    """Merge hand-authored buff entries into the dbuffcollections data.

    Each patch entry is a raw dbuffcollections-shaped mapping with an extra
    `id` key. Hand-authored buffs use negative IDs and must never collide
    with client data.
    """
    for entry in patches:
        buff_id = entry["id"]
        if buff_id >= 0:
            raise ValueError(
                f"Patched buff {buff_id} must use a negative (hand-authored) ID."
            )
        if buff_id in entries:
            raise ValueError(f"Patched buff {buff_id} collides with existing data.")

        entry = dict(entry)
        del entry["id"]
        entry["buffID"] = buff_id
        entries[buff_id] = entry
