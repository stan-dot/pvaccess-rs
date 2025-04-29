#!/bin/bash                                                                                                                                                                         
a=""                                                                                                                                                                                
b=""                                                                                                                                                                                
for i in $(seq 0 60)                                                                                                                                                                
do                                                                                                                                                                                  
    b=$(git diff --shortstat "@{ $i day ago }")                                                                                                                                     
    if [[ "$b" != "$a" ]]; then                                                                                                                                                     
        echo $i "day ago" $b                                                                                                                                                        
    fi                                                                                                                                                                              
    a=$b                                                                                                                                                                            
done 
