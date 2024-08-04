INSERT INTO workspaces (name, owner_id)
VALUES ('workspace1', 0),
    ('workspace2', 0),
    ('workspace3', 0);
-- insert 5 users, all with password 'hunter42'
INSERT INTO users (ws_id, email, fullname, password_hash)
VALUES (
        1,
        'user1@acme.org',
        'user1',
        '$argon2id$v=19$m=19456,t=2,p=1$uSA/eb4JlZTu5o0OOP7axw$aEDst9wVRgHtPaqRySrpuBFuNHK2ncwTl7W7O4dEJ/c'
    ),
    (
        1,
        'user2@acme.org',
        'user2',
        '$argon2id$v=19$m=19456,t=2,p=1$uSA/eb4JlZTu5o0OOP7axw$aEDst9wVRgHtPaqRySrpuBFuNHK2ncwTl7W7O4dEJ/c'
    ),
    (
        1,
        'user3@acme.org',
        'user3',
        '$argon2id$v=19$m=19456,t=2,p=1$uSA/eb4JlZTu5o0OOP7axw$aEDst9wVRgHtPaqRySrpuBFuNHK2ncwTl7W7O4dEJ/c'
    ),
    (
        1,
        'user4@acme.org',
        'user4',
        '$argon2id$v=19$m=19456,t=2,p=1$uSA/eb4JlZTu5o0OOP7axw$aEDst9wVRgHtPaqRySrpuBFuNHK2ncwTl7W7O4dEJ/c'
    ),
    (
        1,
        'user5@acme.org',
        'user5',
        '$argon2id$v=19$m=19456,t=2,p=1$uSA/eb4JlZTu5o0OOP7axw$aEDst9wVRgHtPaqRySrpuBFuNHK2ncwTl7W7O4dEJ/c'
    );
-- insert 4 chats
-- insert public/private channel
INSERT INTO chats (ws_id, name, type, members)
VALUES (
        1,
        'general',
        'public_channel',
        '{1,2,3,4,5}'
    ),
    (
        1,
        'private',
        'private_channel',
        '{1,2,3}'
    );
-- insert unnamed chat
INSERT INTO chats (ws_id, type, members)
VALUES (
        1,
        'single',
        '{1,2}'
    ),
    (
        1,
        'group',
        '{1,3,4}'
    );
