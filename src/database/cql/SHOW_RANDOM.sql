SELECT *
  FROM eve.stablediffusion
    WHERE token(id) > token(?)
    LIMIT 1