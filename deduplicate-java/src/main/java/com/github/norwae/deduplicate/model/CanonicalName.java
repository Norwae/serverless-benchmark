package com.github.norwae.deduplicate.model;

import com.amazonaws.services.dynamodbv2.datamodeling.DynamoDBAttribute;
import com.amazonaws.services.dynamodbv2.datamodeling.DynamoDBHashKey;
import com.amazonaws.services.dynamodbv2.datamodeling.DynamoDBTable;
import lombok.AllArgsConstructor;
import lombok.Data;
import lombok.NoArgsConstructor;

@DynamoDBTable(tableName = "canonicalsJava")
@Data
@AllArgsConstructor
@NoArgsConstructor
public class CanonicalName {
    @DynamoDBHashKey
    private String hashBase64;
    @DynamoDBAttribute
    private String canonicalName;
}
