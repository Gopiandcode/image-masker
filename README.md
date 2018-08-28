# Image Masker
## About
A short sweet application to segment an arbitrary rgba image into distinct rectangular regions based on transparency.

Uses the marching squares algorithm.

Built in rust.

## Usage:
```
Usage:
    image-masker [OPTIONS] IMAGE

Uses marching squares algorithm to segment a binary image into multiple
distinct rectangles. Uses coordinates starting (0,0) at the top left corner.
Returns the result as a list of tuples

positional arguments:
  IMAGE                 The image in any standard format to be processed

optional arguments:
  -h,--help             show this help message and exit
  -o,--output OUTPUT    An optional output image file to render the results to.
```


## Screenshots
Don't you just hate it when people don't include pictures of visual based programs in their README and instead want you to download and run the program to find out what it does?
Yeah. Me too.

### Input
![Input Image](https://github.com/Gopiandcode/image-masker/raw/master/images/input_example.png)
### Output
![Output Image](https://github.com/Gopiandcode/image-masker/raw/master/images/output_image.png)

Sweet and sexy.
