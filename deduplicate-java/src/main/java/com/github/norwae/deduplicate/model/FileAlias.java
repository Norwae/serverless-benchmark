package com.github.norwae.deduplicate.model;

import com.amazonaws.services.dynamodbv2.datamodeling.DynamoDBAttribute;
import com.amazonaws.services.dynamodbv2.datamodeling.DynamoDBHashKey;
import com.amazonaws.services.dynamodbv2.datamodeling.DynamoDBTable;
import lombok.AllArgsConstructor;
import lombok.Data;
import lombok.NoArgsConstructor;

@DynamoDBTable(tableName = "fileAliasJava")
@Data
@AllArgsConstructor
@NoArgsConstructor
public class FileAlias {
    @DynamoDBHashKey
    private String fileName;
    @DynamoDBAttribute
    private String canonicalName;
}
