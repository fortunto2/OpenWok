-- Add 4 new LA pilot zones (extending existing Downtown LA + Hollywood)

INSERT OR IGNORE INTO zones (id, name) VALUES
    ('zone-vnce-0001', 'Venice'),
    ('zone-smca-0001', 'Santa Monica'),
    ('zone-ktwn-0001', 'Koreatown'),
    ('zone-svlk-0001', 'Silver Lake');
