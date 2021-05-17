package com.github.norwae.deduplicate;

import lombok.AllArgsConstructor;
import lombok.Data;

@Data
public class HashedS3Object {

    private final String bucket;
    private final String key;
    private final byte[] sha512;


}
